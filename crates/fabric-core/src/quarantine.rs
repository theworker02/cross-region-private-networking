//! Quarantine: revoke eligibility, drop routes, invalidate ads, keep forensics.

use crate::dns::PrivateDns;
use crate::error::{FabricError, Result};
use crate::ids::NodeID;
use crate::membership::{MemberView, MembershipConsensus};
use crate::observability::{EventLog, FabricEvent};
use crate::receipts::{NetworkReceipt, ReceiptKind, ReceiptLog};
use crate::registry::ServiceRegistry;
use crate::admission::{AdmissionController, AdmissionState};
use crate::identity::TrustStore;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Forensic snapshot retained after quarantine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuarantineForensics {
    /// Node.
    pub node: NodeID,
    /// When quarantined.
    pub at: chrono::DateTime<Utc>,
    /// Reason.
    pub reason: String,
    /// Last known services on node.
    pub services: Vec<String>,
    /// Last peer metrics if any.
    pub observations: Vec<String>,
}

/// Quarantine engine.
#[derive(Default)]
pub struct QuarantineBoard {
    forensics: HashMap<String, QuarantineForensics>,
}

impl QuarantineBoard {
    /// Create.
    pub fn new() -> Self {
        Self::default()
    }

    /// Quarantine a node.
    pub fn quarantine(
        &mut self,
        node: &NodeID,
        reason: impl Into<String>,
        registry: &ServiceRegistry,
        dns: &PrivateDns,
        trust: &mut TrustStore,
        admission: &mut AdmissionController,
        membership: &mut MembershipConsensus,
        events: &EventLog,
        receipts: &mut ReceiptLog,
        sessions_remove: &mut dyn FnMut(&NodeID),
    ) -> Result<QuarantineForensics> {
        let reason = reason.into();
        let services: Vec<String> = registry
            .all()
            .into_iter()
            .filter(|i| &i.node == node)
            .map(|i| i.service.to_string())
            .collect();

        let forensic = QuarantineForensics {
            node: node.clone(),
            at: Utc::now(),
            reason: reason.clone(),
            services: services.clone(),
            observations: vec![format!("services_removed={}", services.len())],
        };

        // Remove routes / registrations
        registry.deregister_node(node);
        dns.invalidate_node(node);
        // Revoke eligibility
        trust.revoke(node);
        let _ = admission.revoke(node, reason.clone());
        sessions_remove(node);

        let epoch = membership
            .get(node)
            .map(|v| v.revocation_epoch + 1)
            .unwrap_or(1);
        membership.observe(MemberView {
            node: node.clone(),
            admission: AdmissionState::Revoked,
            policy_version: 0,
            registry_version: registry.version(),
            revocation_epoch: epoch,
        });

        events.emit(FabricEvent::PeerDown {
            node: node.to_string(),
            reason: format!("quarantine: {reason}"),
        });
        receipts.record(NetworkReceipt::new(
            "control-plane",
            ReceiptKind::Identity {
                node: node.to_string(),
                detail: format!("quarantined: {reason}"),
            },
            forensic.observations.clone(),
        ));

        self.forensics.insert(node.to_string(), forensic.clone());
        Ok(forensic)
    }

    /// Retrieve forensics (preserved after quarantine).
    pub fn forensics(&self, node: &NodeID) -> Option<&QuarantineForensics> {
        self.forensics.get(node.as_str())
    }

    /// Ensure quarantined node cannot be used for routing ads.
    pub fn assert_not_eligible(&self, node: &NodeID, trust: &TrustStore) -> Result<()> {
        if self.forensics.contains_key(node.as_str()) {
            return Err(FabricError::RevokedIdentity(format!(
                "{node} quarantined"
            )));
        }
        if trust.get(node).is_none() {
            // May be revoked without forensic in other paths
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ServiceInstance;
    use chrono::Duration;

    #[test]
    fn quarantine_removes_routes_keeps_forensics() {
        let reg = ServiceRegistry::new();
        reg.register(ServiceInstance::new("api", "bad-node", "frankfurt", "prod", 80));
        let dns = PrivateDns::new(Duration::seconds(30));
        let mut trust = TrustStore::new();
        let id = crate::identity::NodeIdentity::generate(
            "prod".into(),
            "frankfurt".into(),
            Duration::hours(1),
        );
        // Force node id match for test — admit generated then quarantine by that id
        let node = id.public.node_id.clone();
        trust.admit(id.public.clone()).unwrap();
        reg.register(ServiceInstance::new("api", node.as_str(), "frankfurt", "prod", 80));

        let mut adm = AdmissionController::new();
        adm.request_join(id.public.clone());
        adm.authorize(&node, "ok").unwrap();
        let mut mem = MembershipConsensus::new();
        let events = EventLog::new();
        let mut receipts = ReceiptLog::new();
        let mut board = QuarantineBoard::new();
        let mut removed = false;
        board
            .quarantine(
                &node,
                "adversarial route ads",
                &reg,
                &dns,
                &mut trust,
                &mut adm,
                &mut mem,
                &events,
                &mut receipts,
                &mut |n| {
                    if n == &node {
                        removed = true;
                    }
                },
            )
            .unwrap();
        assert!(removed);
        assert!(board.forensics(&node).is_some());
        assert!(reg.lookup_on_node(&"api".into(), &node).is_none());
        assert!(matches!(
            trust.authenticate(&id.public, Utc::now()),
            Err(FabricError::RevokedIdentity(_))
        ));
    }
}
