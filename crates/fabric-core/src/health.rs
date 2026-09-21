//! Health states, hysteresis, and failover metrics.

use crate::ids::{NodeID, ServiceID};
use crate::types::{FailoverMetrics, HealthState};
use chrono::Utc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Hysteresis config against flapping.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthHysteresis {
    /// Consecutive failures before UNREACHABLE.
    pub fail_threshold: u32,
    /// Consecutive successes before returning to HEALTHY from DEGRADED.
    pub recover_threshold: u32,
    /// RTT ms above which HEALTHY → DEGRADED.
    pub degrade_rtt_ms: f64,
    /// Loss above which DEGRADED.
    pub degrade_loss: f64,
}

impl Default for HealthHysteresis {
    fn default() -> Self {
        Self {
            fail_threshold: 3,
            recover_threshold: 2,
            degrade_rtt_ms: 200.0,
            degrade_loss: 0.05,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct Tracker {
    state: HealthState,
    fails: u32,
    successes: u32,
}

/// Health manager for peers and service instances.
#[derive(Default)]
pub struct HealthManager {
    hysteresis: HealthHysteresis,
    peers: RwLock<HashMap<String, Tracker>>,
    services: RwLock<HashMap<String, Tracker>>,
}

impl HealthManager {
    /// Create with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom hysteresis.
    pub fn with_hysteresis(h: HealthHysteresis) -> Self {
        Self {
            hysteresis: h,
            ..Self::default()
        }
    }

    fn key(service: &ServiceID, node: &NodeID) -> String {
        format!("{}@{}", service, node)
    }

    /// Observe a probe result for a service instance.
    pub fn observe_service(
        &self,
        service: &ServiceID,
        node: &NodeID,
        ok: bool,
        rtt_ms: f64,
        loss: f64,
    ) -> HealthState {
        let mut g = self.services.write();
        let t = g.entry(Self::key(service, node)).or_default();
        Self::apply(&self.hysteresis, t, ok, rtt_ms, loss)
    }

    /// Observe peer link health.
    pub fn observe_peer(&self, node: &NodeID, ok: bool, rtt_ms: f64, loss: f64) -> HealthState {
        let mut g = self.peers.write();
        let t = g.entry(node.to_string()).or_default();
        Self::apply(&self.hysteresis, t, ok, rtt_ms, loss)
    }

    /// Current service health or UNKNOWN.
    pub fn service_state(&self, service: &ServiceID, node: &NodeID) -> HealthState {
        self.services
            .read()
            .get(&Self::key(service, node))
            .map(|t| t.state)
            .unwrap_or(HealthState::Unknown)
    }

    /// Peer health or UNKNOWN.
    pub fn peer_state(&self, node: &NodeID) -> HealthState {
        self.peers
            .read()
            .get(node.as_str())
            .map(|t| t.state)
            .unwrap_or(HealthState::Unknown)
    }

    fn apply(
        h: &HealthHysteresis,
        t: &mut Tracker,
        ok: bool,
        rtt_ms: f64,
        loss: f64,
    ) -> HealthState {
        if ok {
            t.fails = 0;
            t.successes += 1;
            if rtt_ms >= h.degrade_rtt_ms || loss >= h.degrade_loss {
                t.state = HealthState::Degraded;
            } else if t.state == HealthState::Degraded || t.state == HealthState::Unreachable {
                if t.successes >= h.recover_threshold {
                    t.state = HealthState::Healthy;
                } else {
                    t.state = HealthState::Degraded;
                }
            } else {
                t.state = HealthState::Healthy;
            }
        } else {
            t.successes = 0;
            t.fails += 1;
            if t.fails >= h.fail_threshold {
                t.state = HealthState::Unreachable;
            } else {
                t.state = HealthState::Degraded;
            }
        }
        t.state
    }

    /// Run a timed failover sequence callback and return metrics.
    pub fn measure_failover<FDetect, FRecalc, FReconnect>(
        &self,
        from: Option<NodeID>,
        detect: FDetect,
        recalc: FRecalc,
        reconnect: FReconnect,
        reason: impl Into<String>,
    ) -> FailoverMetrics
    where
        FDetect: FnOnce(),
        FRecalc: FnOnce() -> Option<NodeID>,
        FReconnect: FnOnce(),
    {
        let t0 = Instant::now();
        detect();
        let t_detect = t0.elapsed();
        let t1 = Instant::now();
        let to = recalc();
        let t_recalc = t1.elapsed();
        let t2 = Instant::now();
        reconnect();
        let t_reconn = t2.elapsed();
        let total = t0.elapsed();
        FailoverMetrics {
            failure_detection_ms: t_detect.as_secs_f64() * 1000.0,
            route_recalculation_ms: t_recalc.as_secs_f64() * 1000.0,
            reconnection_ms: t_reconn.as_secs_f64() * 1000.0,
            total_failover_ms: total.as_secs_f64() * 1000.0,
            from_node: from,
            to_node: to,
            reason: reason.into(),
        }
    }
}

/// Connection drain state (Phase 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DrainState {
    /// Accepting new connections.
    Active,
    /// No new connections; existing finish.
    Draining,
    /// Fully retired.
    Retired,
}

/// Path drain tracker.
#[derive(Clone, Debug, Default)]
pub struct DrainTracker {
    paths: HashMap<String, (DrainState, u32)>,
}

impl DrainTracker {
    /// Mark path draining; returns remaining active conn count.
    pub fn begin_drain(&mut self, path_id: &str, active_conns: u32) {
        self.paths
            .insert(path_id.to_string(), (DrainState::Draining, active_conns));
    }

    /// Finish one connection on a draining path.
    pub fn conn_finished(&mut self, path_id: &str) -> DrainState {
        if let Some((state, n)) = self.paths.get_mut(path_id) {
            if *n > 0 {
                *n -= 1;
            }
            if *n == 0 {
                *state = DrainState::Retired;
            }
            return *state;
        }
        DrainState::Retired
    }

    /// Whether new connections may use this path.
    pub fn accepts_new(&self, path_id: &str) -> bool {
        match self.paths.get(path_id) {
            Some((DrainState::Active, _)) | None => true,
            Some((DrainState::Draining | DrainState::Retired, _)) => false,
        }
    }

    /// Current state.
    pub fn state(&self, path_id: &str) -> DrainState {
        self.paths
            .get(path_id)
            .map(|(s, _)| *s)
            .unwrap_or(DrainState::Active)
    }
}

/// Standby path for fast failover (Phase 3).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StandbyPath {
    /// Primary node.
    pub primary: NodeID,
    /// Precomputed standby.
    pub standby: NodeID,
    /// Service.
    pub service: ServiceID,
    /// When standby was computed.
    pub computed_at: chrono::DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hysteresis_avoids_single_blip() {
        let h = HealthManager::with_hysteresis(HealthHysteresis {
            fail_threshold: 3,
            recover_threshold: 2,
            ..Default::default()
        });
        let s = h.observe_service(&"api".into(), &"n1".into(), false, 10.0, 0.0);
        assert_eq!(s, HealthState::Degraded);
        let s = h.observe_service(&"api".into(), &"n1".into(), false, 10.0, 0.0);
        assert_eq!(s, HealthState::Degraded);
        let s = h.observe_service(&"api".into(), &"n1".into(), false, 10.0, 0.0);
        assert_eq!(s, HealthState::Unreachable);
    }

    #[test]
    fn drain_flow() {
        let mut d = DrainTracker::default();
        d.begin_drain("p1", 2);
        assert!(!d.accepts_new("p1"));
        assert_eq!(d.conn_finished("p1"), DrainState::Draining);
        assert_eq!(d.conn_finished("p1"), DrainState::Retired);
    }
}
