//! Network-only region evacuation (not app/storage migration).

use crate::dns::PrivateDns;
use crate::health::{DrainState, DrainTracker};
use crate::ids::{NodeID, RegionID, ServiceID};
use crate::registry::ServiceRegistry;
use crate::types::HealthState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unresolved dependency after evacuation attempt.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnresolvedDependency {
    /// Service lacking alternate region.
    pub service: ServiceID,
    /// Last known node.
    pub node: NodeID,
    /// Why unresolved.
    pub reason: String,
}

/// Evacuation report (NETWORK scope only).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvacuateReport {
    /// Target region.
    pub region: RegionID,
    /// Dry-run?
    pub dry_run: bool,
    /// Nodes drained.
    pub drained_nodes: Vec<NodeID>,
    /// Services redirected to other regions.
    pub redirected: Vec<(ServiceID, NodeID)>,
    /// Unresolved (no alternate healthy instance).
    pub unresolved: Vec<UnresolvedDependency>,
    /// Timestamp.
    pub at: DateTime<Utc>,
    /// Scope disclaimer.
    pub scope_note: String,
}

/// Evacuate a region from the **network fabric** perspective.
///
/// Does **not** migrate application data, volumes, or perform deployment cutover.
pub fn evacuate_region(
    region: &RegionID,
    registry: &ServiceRegistry,
    dns: &PrivateDns,
    drain: &mut DrainTracker,
    dry_run: bool,
) -> EvacuateReport {
    let mut drained_nodes = Vec::new();
    let mut redirected = Vec::new();
    let mut unresolved = Vec::new();

    let all = registry.all();
    let in_region: Vec<_> = all.iter().filter(|i| &i.region == region).cloned().collect();

    for inst in &in_region {
        if !drained_nodes.contains(&inst.node) {
            drained_nodes.push(inst.node.clone());
        }
        // Find alternate in another region
        let alts: Vec<_> = registry
            .lookup(&inst.service)
            .into_iter()
            .filter(|a| &a.region != region && a.health.is_routable())
            .collect();
        if let Some(alt) = alts.first() {
            redirected.push((inst.service.clone(), alt.node.clone()));
            if !dry_run {
                registry.update_health(&inst.service, &inst.node, HealthState::Unreachable);
                dns.on_health_failure(&inst.service, &inst.node);
                drain.begin_drain(inst.node.as_str(), 1);
                let _ = drain.conn_finished(inst.node.as_str());
                let _ = DrainState::Retired;
            }
        } else {
            unresolved.push(UnresolvedDependency {
                service: inst.service.clone(),
                node: inst.node.clone(),
                reason: format!(
                    "no healthy alternate instance outside region {region} (network-only evacuate)"
                ),
            });
            if !dry_run {
                registry.update_health(&inst.service, &inst.node, HealthState::Unreachable);
                dns.on_health_failure(&inst.service, &inst.node);
                drain.begin_drain(inst.node.as_str(), 0);
            }
        }
    }

    EvacuateReport {
        region: region.clone(),
        dry_run,
        drained_nodes,
        redirected,
        unresolved,
        at: Utc::now(),
        scope_note: "NETWORK ONLY — no application/storage migration, volume replication, \
            or deployment cutover. Unresolved dependencies require operators to deploy \
            alternate instances or accept outage."
            .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ServiceInstance;
    use chrono::Duration;

    #[test]
    fn dry_run_does_not_mutate_health() {
        let reg = ServiceRegistry::new();
        reg.register(ServiceInstance::new(
            "api", "n-fra", "frankfurt", "prod", 80,
        ));
        let dns = PrivateDns::new(Duration::seconds(30));
        let mut drain = DrainTracker::default();
        let report = evacuate_region(
            &"frankfurt".into(),
            &reg,
            &dns,
            &mut drain,
            true,
        );
        assert!(report.dry_run);
        assert_eq!(
            reg.lookup(&"api".into())[0].health,
            HealthState::Healthy
        );
        assert!(!report.unresolved.is_empty());
    }

    #[test]
    fn redirects_when_alternate_exists() {
        let reg = ServiceRegistry::new();
        reg.register(ServiceInstance::new(
            "api", "n-fra", "frankfurt", "prod", 80,
        ));
        reg.register(ServiceInstance::new(
            "api", "n-va", "virginia", "prod", 80,
        ));
        let dns = PrivateDns::new(Duration::seconds(30));
        let mut drain = DrainTracker::default();
        let report = evacuate_region(
            &"frankfurt".into(),
            &reg,
            &dns,
            &mut drain,
            false,
        );
        assert!(!report.redirected.is_empty());
        assert!(report.unresolved.is_empty());
    }
}
