//! Topology modes and self-healing edge replacement.

use crate::ids::{NodeID, RegionID};
use crate::types::{HealthState, LinkMetrics, TopologyMode};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// An undirected-as-bidirectional peering edge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TopologyEdge {
    /// Endpoint A.
    pub a: RegionID,
    /// Endpoint B.
    pub b: RegionID,
    /// Optional relay node id if edge is relayed.
    pub via_relay: Option<NodeID>,
}

/// Computed topology plan.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TopologyPlan {
    /// Mode used.
    pub mode: TopologyMode,
    /// Edges in the plan.
    pub edges: Vec<TopologyEdge>,
    /// Explanation.
    pub reason: String,
}

/// Topology engine — FULL_MESH, HUB_SPOKE, DYNAMIC (deterministic).
pub struct TopologyEngine;

impl TopologyEngine {
    /// Compute edges for regions given mode and optional metrics.
    pub fn compute(
        mode: TopologyMode,
        regions: &[RegionID],
        hub: Option<&RegionID>,
        links: &[LinkMetrics],
    ) -> TopologyPlan {
        match mode {
            TopologyMode::FullMesh => {
                let mut edges = Vec::new();
                for i in 0..regions.len() {
                    for j in (i + 1)..regions.len() {
                        edges.push(TopologyEdge {
                            a: regions[i].clone(),
                            b: regions[j].clone(),
                            via_relay: None,
                        });
                    }
                }
                TopologyPlan {
                    mode,
                    edges,
                    reason: format!("FULL_MESH n={} edges={}", regions.len(), edges_count(regions.len())),
                }
            }
            TopologyMode::HubSpoke => {
                let hub_r = hub
                    .cloned()
                    .unwrap_or_else(|| regions.first().cloned().unwrap_or_else(|| RegionID::new("hub")));
                let mut edges = Vec::new();
                for r in regions {
                    if r != &hub_r {
                        edges.push(TopologyEdge {
                            a: hub_r.clone(),
                            b: r.clone(),
                            via_relay: None,
                        });
                    }
                }
                TopologyPlan {
                    mode,
                    edges,
                    reason: format!("HUB_SPOKE hub={}", hub_r),
                }
            }
            TopologyMode::Dynamic => dynamic_plan(regions, links),
        }
    }

    /// Self-heal: replace unhealthy edges, validate connectivity, return new plan.
    pub fn heal(
        current: &TopologyPlan,
        regions: &[RegionID],
        links: &[LinkMetrics],
        hub: Option<&RegionID>,
    ) -> TopologyPlan {
        let bad: HashSet<(String, String)> = links
            .iter()
            .filter(|l| l.health == HealthState::Unreachable || l.loss > 0.2)
            .map(|l| edge_key(&l.from, &l.to))
            .collect();

        let healthy_edges: Vec<TopologyEdge> = current
            .edges
            .iter()
            .filter(|e| {
                !bad.contains(&edge_key(&e.a, &e.b)) && !bad.contains(&edge_key(&e.b, &e.a))
            })
            .cloned()
            .collect();

        // If graph disconnected, recompute DYNAMIC.
        if !is_connected(regions, &healthy_edges) {
            let mut plan = Self::compute(TopologyMode::Dynamic, regions, hub, links);
            plan.reason = format!(
                "self-heal: detected partition; recomputed DYNAMIC ({})",
                plan.reason
            );
            return plan;
        }

        // Replace missing pairs that had demand with best alternate via hub or lowest latency.
        let mut edges = healthy_edges;
        for link in links {
            if link.health.is_routable()
                && link.traffic_demand > 0.5
                && !edges.iter().any(|e| same_edge(e, &link.from, &link.to))
            {
                edges.push(TopologyEdge {
                    a: link.from.clone(),
                    b: link.to.clone(),
                    via_relay: None,
                });
            }
        }
        TopologyPlan {
            mode: current.mode,
            edges,
            reason: "self-heal: pruned unhealthy edges; added demand links".into(),
        }
    }
}

fn edges_count(n: usize) -> usize {
    n.saturating_sub(1) * n / 2
}

fn edge_key(a: &RegionID, b: &RegionID) -> (String, String) {
    let (x, y) = if a.as_str() <= b.as_str() {
        (a.as_str(), b.as_str())
    } else {
        (b.as_str(), a.as_str())
    };
    (x.to_string(), y.to_string())
}

fn same_edge(e: &TopologyEdge, a: &RegionID, b: &RegionID) -> bool {
    (&e.a == a && &e.b == b) || (&e.a == b && &e.b == a)
}

fn is_connected(regions: &[RegionID], edges: &[TopologyEdge]) -> bool {
    if regions.is_empty() {
        return true;
    }
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for r in regions {
        adj.entry(r.to_string()).or_default();
    }
    for e in edges {
        adj.entry(e.a.to_string())
            .or_default()
            .push(e.b.to_string());
        adj.entry(e.b.to_string())
            .or_default()
            .push(e.a.to_string());
    }
    let start = regions[0].to_string();
    let mut seen = HashSet::new();
    let mut stack = vec![start];
    while let Some(n) = stack.pop() {
        if seen.insert(n.clone()) {
            if let Some(nei) = adj.get(&n) {
                for x in nei {
                    stack.push(x.clone());
                }
            }
        }
    }
    seen.len() == regions.len()
}

/// Deterministic DYNAMIC topology from region count, latency, health, demand, overhead.
fn dynamic_plan(regions: &[RegionID], links: &[LinkMetrics]) -> TopologyPlan {
    // Score candidate edges; take MST-like backbone then add high-demand links under overhead budget.
    let mut candidates: Vec<(f64, RegionID, RegionID)> = Vec::new();
    for i in 0..regions.len() {
        for j in (i + 1)..regions.len() {
            let a = &regions[i];
            let b = &regions[j];
            let m = links.iter().find(|l| {
                (&l.from == a && &l.to == b) || (&l.from == b && &l.to == a)
            });
            let (lat, loss, health, demand) = match m {
                Some(l) => (l.latency_ms, l.loss, l.health, l.traffic_demand),
                None => (150.0, 0.02, HealthState::Unknown, 0.1),
            };
            if health == HealthState::Unreachable {
                continue;
            }
            // Lower score = better edge for backbone.
            let score = lat + loss * 500.0
                + if health == HealthState::Degraded { 50.0 } else { 0.0 }
                - demand * 20.0;
            candidates.push((score, a.clone(), b.clone()));
        }
    }
    candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Kruskal-like MST
    let mut parent: HashMap<String, String> =
        regions.iter().map(|r| (r.to_string(), r.to_string())).collect();
    fn find(p: &mut HashMap<String, String>, x: &str) -> String {
        let par = p[x].clone();
        if par == x {
            return par;
        }
        let root = find(p, &par);
        p.insert(x.to_string(), root.clone());
        root
    }
    let mut edges = Vec::new();
    for (score, a, b) in &candidates {
        let ra = find(&mut parent, a.as_str());
        let rb = find(&mut parent, b.as_str());
        if ra != rb {
            parent.insert(ra, rb);
            edges.push(TopologyEdge {
                a: a.clone(),
                b: b.clone(),
                via_relay: None,
            });
            let _ = score;
        }
    }
    // Connection overhead budget: allow up to n extra high-demand edges.
    let budget = regions.len().max(1);
    let mut added = 0;
    for (_score, a, b) in &candidates {
        if added >= budget {
            break;
        }
        if edges.iter().any(|e| same_edge(e, a, b)) {
            continue;
        }
        let demand = links
            .iter()
            .find(|l| (&l.from == a && &l.to == b) || (&l.from == b && &l.to == a))
            .map(|l| l.traffic_demand)
            .unwrap_or(0.0);
        if demand >= 0.7 {
            edges.push(TopologyEdge {
                a: a.clone(),
                b: b.clone(),
                via_relay: None,
            });
            added += 1;
        }
    }
    TopologyPlan {
        mode: TopologyMode::Dynamic,
        edges: edges.clone(),
        reason: format!(
            "DYNAMIC deterministic MST+demand regions={} edges={}",
            regions.len(),
            edges.len()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_mesh_three() {
        let regions = vec!["a".into(), "b".into(), "c".into()];
        let p = TopologyEngine::compute(TopologyMode::FullMesh, &regions, None, &[]);
        assert_eq!(p.edges.len(), 3);
    }

    #[test]
    fn dynamic_deterministic() {
        let regions = vec!["r1".into(), "r2".into(), "r3".into(), "r4".into()];
        let links = vec![
            LinkMetrics {
                from: "r1".into(),
                to: "r2".into(),
                latency_ms: 10.0,
                loss: 0.0,
                health: HealthState::Healthy,
                traffic_demand: 1.0,
            },
            LinkMetrics {
                from: "r2".into(),
                to: "r3".into(),
                latency_ms: 12.0,
                loss: 0.0,
                health: HealthState::Healthy,
                traffic_demand: 0.2,
            },
            LinkMetrics {
                from: "r3".into(),
                to: "r4".into(),
                latency_ms: 11.0,
                loss: 0.0,
                health: HealthState::Healthy,
                traffic_demand: 0.2,
            },
            LinkMetrics {
                from: "r1".into(),
                to: "r4".into(),
                latency_ms: 80.0,
                loss: 0.0,
                health: HealthState::Healthy,
                traffic_demand: 0.9,
            },
        ];
        let p1 = TopologyEngine::compute(TopologyMode::Dynamic, &regions, None, &links);
        let p2 = TopologyEngine::compute(TopologyMode::Dynamic, &regions, None, &links);
        assert_eq!(p1.edges, p2.edges);
        assert!(p1.edges.len() >= 3);
    }
}
