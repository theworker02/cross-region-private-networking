//! Dynamic weighted network graph + active path measurement.

use crate::ids::{NodeID, RegionID, ServiceID};
use crate::types::HealthState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Provenance of a metric sample — never report ESTIMATED as OBSERVED.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricSource {
    /// Actively probed / measured.
    Observed,
    /// Heuristic or last-known decay.
    Estimated,
    /// No data.
    Unknown,
}

impl MetricSource {
    /// Label.
    pub fn as_str(self) -> &'static str {
        match self {
            MetricSource::Observed => "OBSERVED",
            MetricSource::Estimated => "ESTIMATED",
            MetricSource::Unknown => "UNKNOWN",
        }
    }
}

/// Edge attributes between graph nodes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphEdge {
    /// From node id (fabric node or region gateway).
    pub from: NodeID,
    /// To node id.
    pub to: NodeID,
    /// RTT ms.
    pub rtt_ms: f64,
    /// Jitter ms.
    pub jitter_ms: f64,
    /// Loss 0..1.
    pub loss: f64,
    /// Throughput hint Mbps.
    pub throughput_mbps: f64,
    /// Health.
    pub health: HealthState,
    /// Stability score 0..1 (higher better).
    pub stability: f64,
    /// Relative cost.
    pub cost: f64,
    /// Metric source.
    pub source: MetricSource,
    /// Last observation time.
    pub last_observation: Option<DateTime<Utc>>,
}

/// Graph vertex kinds.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VertexKind {
    /// Fabric node.
    Node {
        /// Region.
        region: RegionID,
    },
    /// Region aggregate.
    Region,
    /// Relay.
    Relay,
    /// Service attachment.
    Service {
        /// Service id.
        service: ServiceID,
    },
}

/// Network graph engine.
#[derive(Clone, Debug, Default)]
pub struct NetworkGraph {
    vertices: HashMap<String, VertexKind>,
    edges: HashMap<(String, String), GraphEdge>,
}

impl NetworkGraph {
    /// Create.
    pub fn new() -> Self {
        Self::default()
    }

    /// Upsert vertex.
    pub fn upsert_vertex(&mut self, id: &str, kind: VertexKind) {
        self.vertices.insert(id.to_string(), kind);
    }

    /// Record an **observed** probe (preferred over synthetic).
    pub fn record_probe(
        &mut self,
        from: &NodeID,
        to: &NodeID,
        rtt_ms: f64,
        jitter_ms: f64,
        loss: f64,
        throughput_mbps: f64,
    ) {
        let stability = (1.0 - loss).clamp(0.0, 1.0) * if rtt_ms < 100.0 { 1.0 } else { 0.8 };
        let health = if loss > 0.2 || rtt_ms > 1000.0 {
            HealthState::Unreachable
        } else if loss > 0.05 || rtt_ms > 250.0 {
            HealthState::Degraded
        } else {
            HealthState::Healthy
        };
        let edge = GraphEdge {
            from: from.clone(),
            to: to.clone(),
            rtt_ms,
            jitter_ms,
            loss,
            throughput_mbps,
            health,
            stability,
            cost: rtt_ms / 10.0 + loss * 50.0,
            source: MetricSource::Observed,
            last_observation: Some(Utc::now()),
        };
        self.edges
            .insert((from.to_string(), to.to_string()), edge);
    }

    /// Insert estimated edge (explicitly labeled).
    pub fn set_estimated(&mut self, from: &NodeID, to: &NodeID, rtt_ms: f64) {
        self.edges.insert(
            (from.to_string(), to.to_string()),
            GraphEdge {
                from: from.clone(),
                to: to.clone(),
                rtt_ms,
                jitter_ms: rtt_ms * 0.1,
                loss: 0.02,
                throughput_mbps: 100.0,
                health: HealthState::Unknown,
                stability: 0.5,
                cost: rtt_ms / 8.0,
                source: MetricSource::Estimated,
                last_observation: None,
            },
        );
    }

    /// Get edge.
    pub fn edge(&self, from: &NodeID, to: &NodeID) -> Option<&GraphEdge> {
        self.edges.get(&(from.to_string(), to.to_string()))
    }

    /// All edges.
    pub fn edges(&self) -> impl Iterator<Item = &GraphEdge> {
        self.edges.values()
    }

    /// Prefer OBSERVED edges when ranking paths; skip UNKNOWN health for primary.
    pub fn best_next_hop(&self, from: &NodeID, candidates: &[NodeID]) -> Option<(NodeID, GraphEdge)> {
        let mut best: Option<(NodeID, GraphEdge)> = None;
        for c in candidates {
            if let Some(e) = self.edge(from, c) {
                if e.health == HealthState::Unreachable {
                    continue;
                }
                let better = match &best {
                    None => true,
                    Some((_, cur)) => {
                        // Prefer observed over estimated.
                        match (e.source, cur.source) {
                            (MetricSource::Observed, MetricSource::Estimated) => true,
                            (MetricSource::Estimated, MetricSource::Observed) => false,
                            _ => e.rtt_ms < cur.rtt_ms,
                        }
                    }
                };
                if better {
                    best = Some((c.clone(), e.clone()));
                }
            }
        }
        best
    }
}

/// Lightweight continuous probe scheduler (logical; tests drive ticks).
#[derive(Clone, Debug, Default)]
pub struct ProbeScheduler {
    /// Probe interval hint ms.
    pub interval_ms: u64,
}

impl ProbeScheduler {
    /// Default 5s.
    pub fn new() -> Self {
        Self {
            interval_ms: 5_000,
        }
    }

    /// Simulate one probe round between peers; returns observed RTT samples.
    pub fn probe_round(
        &self,
        graph: &mut NetworkGraph,
        pairs: &[(NodeID, NodeID, f64)],
    ) -> Vec<(NodeID, NodeID, f64, MetricSource)> {
        let mut out = Vec::new();
        for (a, b, true_rtt) in pairs {
            // Lightweight: record as OBSERVED (harness supplies true_rtt).
            graph.record_probe(a, b, *true_rtt, true_rtt * 0.05, 0.0, 500.0);
            out.push((a.clone(), b.clone(), *true_rtt, MetricSource::Observed));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_confuses_estimated_with_observed() {
        let mut g = NetworkGraph::new();
        g.set_estimated(&"a".into(), &"b".into(), 40.0);
        assert_eq!(
            g.edge(&"a".into(), &"b".into()).unwrap().source,
            MetricSource::Estimated
        );
        g.record_probe(&"a".into(), &"b".into(), 35.0, 2.0, 0.0, 400.0);
        assert_eq!(
            g.edge(&"a".into(), &"b".into()).unwrap().source,
            MetricSource::Observed
        );
    }
}
