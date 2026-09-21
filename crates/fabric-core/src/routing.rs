//! Topology-aware routing with explainability (Phase 2) and multi-path policies (Phase 3).

use crate::error::{FabricError, Result};
use crate::ids::{NodeID, RegionID, ServiceID};
use crate::types::{HealthState, ServiceInstance, TransportMode};
use serde::{Deserialize, Serialize};

/// Path selection policy (Phase 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PathPolicy {
    /// Minimize observed/estimated RTT.
    #[default]
    LowestLatency,
    /// Prefer low loss + high stability.
    MostStable,
    /// Prefer a preferred region when healthy.
    RegionPreferred,
    /// Prefer lower cost edges.
    CostAware,
    /// Weighted blend of latency, stability, cost, locality.
    Balanced,
}

/// Scored alternative for explainability.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RouteAlternative {
    /// Candidate node.
    pub node: NodeID,
    /// Region.
    pub region: RegionID,
    /// Composite score (lower is better).
    pub score: f64,
    /// Why not chosen (or runner-up note).
    pub note: String,
}

/// Full routing decision with explainability.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RouteDecision {
    /// Chosen instance.
    pub instance: ServiceInstance,
    /// Transport mode for this path.
    pub transport: TransportMode,
    /// Composite score (lower better).
    pub score: f64,
    /// Latency component used in scoring (ms).
    pub score_latency_ms: f64,
    /// Human-readable reason.
    pub reason: String,
    /// Alternatives considered.
    pub alternatives: Vec<RouteAlternative>,
}

/// Flap-control configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlapControl {
    /// Minimum score improvement required to switch (absolute).
    pub min_improvement: f64,
    /// Minimum path lifetime before voluntary switch (ms).
    pub min_lifetime_ms: f64,
    /// Hysteresis multiplier on current path score (current treated as better by this factor).
    pub hysteresis: f64,
}

impl Default for FlapControl {
    fn default() -> Self {
        Self {
            min_improvement: 5.0,
            min_lifetime_ms: 1_000.0,
            hysteresis: 1.15,
        }
    }
}

/// Routing engine.
#[derive(Clone, Debug)]
pub struct RoutingEngine {
    /// Active path policy.
    pub policy: PathPolicy,
    /// Preferred region for REGION_PREFERRED / locality boost.
    pub preferred_region: Option<RegionID>,
    /// Flap control knobs.
    pub flap: FlapControl,
    /// Sticky affinity: client_key → node.
    affinity: std::collections::HashMap<String, (NodeID, i64)>,
    /// Current active path per service (for hysteresis).
    active: std::collections::HashMap<String, (NodeID, i64, f64)>,
}

impl Default for RoutingEngine {
    fn default() -> Self {
        Self {
            policy: PathPolicy::Balanced,
            preferred_region: None,
            flap: FlapControl::default(),
            affinity: Default::default(),
            active: Default::default(),
        }
    }
}

impl RoutingEngine {
    /// Create with policy.
    pub fn with_policy(policy: PathPolicy) -> Self {
        Self {
            policy,
            ..Self::default()
        }
    }

    /// Pin affinity for a client identity key to a service instance node.
    pub fn set_affinity(&mut self, client_key: &str, node: NodeID) {
        self.affinity
            .insert(client_key.to_string(), (node, chrono::Utc::now().timestamp_millis()));
    }

    /// Clear affinity.
    pub fn clear_affinity(&mut self, client_key: &str) {
        self.affinity.remove(client_key);
    }

    /// Choose best instance among candidates.
    pub fn choose(
        &self,
        service: &ServiceID,
        candidates: &[ServiceInstance],
        local_region: &RegionID,
        force_region: Option<&RegionID>,
    ) -> Result<RouteDecision> {
        self.choose_with_affinity(service, candidates, local_region, force_region, None)
    }

    /// Choose with optional client affinity key.
    pub fn choose_with_affinity(
        &self,
        service: &ServiceID,
        candidates: &[ServiceInstance],
        local_region: &RegionID,
        force_region: Option<&RegionID>,
        client_key: Option<&str>,
    ) -> Result<RouteDecision> {
        let mut eligible: Vec<&ServiceInstance> = candidates
            .iter()
            .filter(|c| c.health.is_routable())
            .filter(|c| force_region.map(|r| &c.region == r).unwrap_or(true))
            .collect();

        if eligible.is_empty() {
            return Err(FabricError::NoRoute(format!(
                "no healthy instances for {service}"
            )));
        }

        // Affinity stickiness (network/service identity only).
        if let Some(key) = client_key {
            if let Some((node, _)) = self.affinity.get(key) {
                if let Some(stuck) = eligible.iter().find(|c| &c.node == node) {
                    return Ok(RouteDecision {
                        instance: (*stuck).clone(),
                        transport: TransportMode::Direct,
                        score: 0.0,
                        score_latency_ms: stuck.latency_ms,
                        reason: format!("service affinity sticky → {}", node),
                        alternatives: vec![],
                    });
                }
            }
        }

        let pref = self
            .preferred_region
            .as_ref()
            .unwrap_or(local_region);

        let mut scored: Vec<(f64, &ServiceInstance, String)> = eligible
            .iter()
            .map(|c| {
                let (score, why) = self.score_instance(c, local_region, pref);
                (score, *c, why)
            })
            .collect();
        scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Flap control vs active path.
        let now = chrono::Utc::now().timestamp_millis();
        let svc_key = service.as_str().to_string();
        if let Some((active_node, since, active_score)) = self.active.get(&svc_key) {
            let age = (now - since) as f64;
            if age < self.flap.min_lifetime_ms {
                if let Some((_, inst, _)) = scored.iter().find(|(_, i, _)| &i.node == active_node) {
                    // Keep unless active became unroutable (already filtered) or failure override.
                    let alts = scored
                        .iter()
                        .skip(1)
                        .take(3)
                        .map(|(s, i, n)| RouteAlternative {
                            node: i.node.clone(),
                            region: i.region.clone(),
                            score: *s,
                            note: n.clone(),
                        })
                        .collect();
                    return Ok(RouteDecision {
                        instance: (*inst).clone(),
                        transport: TransportMode::Direct,
                        score: *active_score,
                        score_latency_ms: inst.latency_ms,
                        reason: format!(
                            "flap control: min lifetime not elapsed ({age:.0}ms < {}ms)",
                            self.flap.min_lifetime_ms
                        ),
                        alternatives: alts,
                    });
                }
            }
            // Require min improvement beyond hysteresis.
            if let Some((best_score, best, _)) = scored.first() {
                let gated = active_score / self.flap.hysteresis;
                if best.node != *active_node
                    && (gated - best_score) < self.flap.min_improvement
                {
                    if let Some((_, inst, _)) =
                        scored.iter().find(|(_, i, _)| &i.node == active_node)
                    {
                        return Ok(RouteDecision {
                            instance: (*inst).clone(),
                            transport: TransportMode::Direct,
                            score: *active_score,
                            score_latency_ms: inst.latency_ms,
                            reason: format!(
                                "flap control: improvement {:.1} < min {}",
                                gated - best_score,
                                self.flap.min_improvement
                            ),
                            alternatives: vec![RouteAlternative {
                                node: best.node.clone(),
                                region: best.region.clone(),
                                score: *best_score,
                                note: "candidate blocked by flap control".into(),
                            }],
                        });
                    }
                }
            }
        }

        let (best_score, best, why) = scored[0].clone();
        let alts: Vec<RouteAlternative> = scored
            .iter()
            .skip(1)
            .take(5)
            .map(|(s, i, n)| RouteAlternative {
                node: i.node.clone(),
                region: i.region.clone(),
                score: *s,
                note: n.clone(),
            })
            .collect();

        Ok(RouteDecision {
            instance: best.clone(),
            transport: TransportMode::Direct,
            score: best_score,
            score_latency_ms: best.latency_ms,
            reason: why,
            alternatives: alts,
        })
    }

    /// Record active path after successful selection (enables flap control).
    pub fn commit_active(&mut self, service: &ServiceID, node: NodeID, score: f64) {
        self.active.insert(
            service.as_str().to_string(),
            (node, chrono::Utc::now().timestamp_millis(), score),
        );
    }

    /// Force failover override (ignores min lifetime / improvement).
    pub fn clear_active(&mut self, service: &ServiceID) {
        self.active.remove(service.as_str());
    }

    fn score_instance(
        &self,
        c: &ServiceInstance,
        local_region: &RegionID,
        pref: &RegionID,
    ) -> (f64, String) {
        let local_boost = if &c.region == local_region { 0.0 } else { 25.0 };
        let health_penalty = match c.health {
            HealthState::Healthy => 0.0,
            HealthState::Degraded => 40.0,
            HealthState::Unknown => 80.0,
            HealthState::Unreachable => 10_000.0,
        };
        let loss_penalty = c.loss * 200.0;
        let latency = c.latency_ms;
        let region_penalty = if &c.region == pref { 0.0 } else { 15.0 };
        // Cost proxy: cross-region + loss.
        let cost = local_boost + loss_penalty * 0.5;

        let (score, label) = match self.policy {
            PathPolicy::LowestLatency => (
                latency + health_penalty,
                format!("LOWEST_LATENCY latency={latency:.1}ms health={:?}", c.health),
            ),
            PathPolicy::MostStable => (
                loss_penalty * 2.0 + health_penalty + latency * 0.1,
                format!("MOST_STABLE loss={:.3} health={:?}", c.loss, c.health),
            ),
            PathPolicy::RegionPreferred => (
                region_penalty + latency + health_penalty,
                format!("REGION_PREFERRED pref={} got={}", pref, c.region),
            ),
            PathPolicy::CostAware => (
                cost + health_penalty + latency * 0.2,
                format!("COST_AWARE cost≈{cost:.1}"),
            ),
            PathPolicy::Balanced => (
                latency + local_boost + health_penalty + loss_penalty + region_penalty * 0.5,
                format!(
                    "BALANCED latency={latency:.1} local={} health={:?} loss={:.3}",
                    &c.region == local_region,
                    c.health,
                    c.loss
                ),
            ),
        };
        (score, label)
    }

    /// Explain a decision as structured text lines.
    pub fn explain(decision: &RouteDecision) -> Vec<String> {
        let mut lines = vec![
            format!(
                "selected: {} @ {} (region {})",
                decision.instance.service, decision.instance.node, decision.instance.region
            ),
            format!("transport: {}", decision.transport.as_str()),
            format!("score: {:.2}", decision.score),
            format!("latency_ms: {:.1}", decision.score_latency_ms),
            format!("health: {:?}", decision.instance.health),
            format!("reason: {}", decision.reason),
        ];
        if decision.alternatives.is_empty() {
            lines.push("alternatives: (none)".into());
        } else {
            lines.push("alternatives:".into());
            for a in &decision.alternatives {
                lines.push(format!(
                    "  - {} @ {} score={:.2} ({})",
                    a.node, a.region, a.score, a.note
                ));
            }
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_local_healthy() {
        let eng = RoutingEngine::with_policy(PathPolicy::Balanced);
        let local = ServiceInstance::new("api", "n1", "us-east", "prod", 80);
        let mut remote = ServiceInstance::new("api", "n2", "eu-central", "prod", 80);
        remote.latency_ms = 90.0;
        let d = eng
            .choose(
                &"api".into(),
                &[local, remote],
                &"us-east".into(),
                None,
            )
            .unwrap();
        assert_eq!(d.instance.node.as_str(), "n1");
    }

    #[test]
    fn affinity_sticks() {
        let mut eng = RoutingEngine::default();
        eng.set_affinity("client-a", "n2".into());
        let a = ServiceInstance::new("api", "n1", "us-east", "prod", 80);
        let mut b = ServiceInstance::new("api", "n2", "eu-central", "prod", 80);
        b.latency_ms = 90.0;
        let d = eng
            .choose_with_affinity(
                &"api".into(),
                &[a, b],
                &"us-east".into(),
                None,
                Some("client-a"),
            )
            .unwrap();
        assert_eq!(d.instance.node.as_str(), "n2");
    }
}
