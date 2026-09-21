//! Admission control: PENDING → AUTHORIZED → ACTIVE → REVOKED.

use crate::error::{FabricError, Result};
use crate::identity::PublicIdentity;
use crate::ids::NodeID;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Admission lifecycle state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdmissionState {
    /// Join requested; not yet approved.
    Pending,
    /// Cryptographically bound and approved; may handshake.
    Authorized,
    /// Actively participating in the fabric.
    Active,
    /// Revoked — cannot rejoin with same identity without re-admission.
    Revoked,
}

/// Admission receipt (audit).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdmissionReceipt {
    /// Node.
    pub node: NodeID,
    /// State after decision.
    pub state: AdmissionState,
    /// Fingerprint of verifying key.
    pub key_fingerprint: String,
    /// When.
    pub at: DateTime<Utc>,
    /// Reason / operator note.
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Record {
    identity: PublicIdentity,
    state: AdmissionState,
}

/// Admission controller — config alone is never enough; crypto identity required.
#[derive(Clone, Debug, Default)]
pub struct AdmissionController {
    records: HashMap<String, Record>,
    receipts: Vec<AdmissionReceipt>,
}

impl AdmissionController {
    /// Create.
    pub fn new() -> Self {
        Self::default()
    }

    /// Submit join request → PENDING.
    pub fn request_join(&mut self, identity: PublicIdentity) -> AdmissionReceipt {
        let node = identity.node_id.clone();
        let fp = identity.fingerprint();
        self.records.insert(
            node.to_string(),
            Record {
                identity,
                state: AdmissionState::Pending,
            },
        );
        let r = AdmissionReceipt {
            node,
            state: AdmissionState::Pending,
            key_fingerprint: fp,
            at: Utc::now(),
            reason: "join requested; awaiting authorization".into(),
        };
        self.receipts.push(r.clone());
        r
    }

    /// Authorize a pending node (operator / control plane).
    pub fn authorize(&mut self, node: &NodeID, reason: impl Into<String>) -> Result<AdmissionReceipt> {
        let rec = self
            .records
            .get_mut(node.as_str())
            .ok_or_else(|| FabricError::UnknownIdentity(node.to_string()))?;
        if rec.state == AdmissionState::Revoked {
            return Err(FabricError::RevokedIdentity(node.to_string()));
        }
        rec.state = AdmissionState::Authorized;
        let r = AdmissionReceipt {
            node: node.clone(),
            state: AdmissionState::Authorized,
            key_fingerprint: rec.identity.fingerprint(),
            at: Utc::now(),
            reason: reason.into(),
        };
        self.receipts.push(r.clone());
        Ok(r)
    }

    /// Mark active after successful handshake.
    pub fn mark_active(&mut self, node: &NodeID) -> Result<AdmissionReceipt> {
        let rec = self
            .records
            .get_mut(node.as_str())
            .ok_or_else(|| FabricError::UnknownIdentity(node.to_string()))?;
        match rec.state {
            AdmissionState::Authorized | AdmissionState::Active => {
                rec.state = AdmissionState::Active;
            }
            AdmissionState::Pending => {
                return Err(FabricError::AuthFailed(
                    "node pending; not yet authorized".into(),
                ));
            }
            AdmissionState::Revoked => {
                return Err(FabricError::RevokedIdentity(node.to_string()));
            }
        }
        let r = AdmissionReceipt {
            node: node.clone(),
            state: AdmissionState::Active,
            key_fingerprint: rec.identity.fingerprint(),
            at: Utc::now(),
            reason: "handshake completed; node active".into(),
        };
        self.receipts.push(r.clone());
        Ok(r)
    }

    /// Revoke — blocks rejoin with same identity.
    pub fn revoke(&mut self, node: &NodeID, reason: impl Into<String>) -> Result<AdmissionReceipt> {
        let rec = self
            .records
            .get_mut(node.as_str())
            .ok_or_else(|| FabricError::UnknownIdentity(node.to_string()))?;
        rec.state = AdmissionState::Revoked;
        let r = AdmissionReceipt {
            node: node.clone(),
            state: AdmissionState::Revoked,
            key_fingerprint: rec.identity.fingerprint(),
            at: Utc::now(),
            reason: reason.into(),
        };
        self.receipts.push(r.clone());
        Ok(r)
    }

    /// Current state.
    pub fn state(&self, node: &NodeID) -> Option<AdmissionState> {
        self.records.get(node.as_str()).map(|r| r.state)
    }

    /// Whether node may establish sessions.
    pub fn may_handshake(&self, node: &NodeID) -> bool {
        matches!(
            self.state(node),
            Some(AdmissionState::Authorized | AdmissionState::Active)
        )
    }

    /// Receipts.
    pub fn receipts(&self) -> &[AdmissionReceipt] {
        &self.receipts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::NodeIdentity;
    use chrono::Duration;

    #[test]
    fn pending_cannot_handshake_until_authorized() {
        let id = NodeIdentity::generate("n".into(), "r".into(), Duration::hours(1));
        let mut ac = AdmissionController::new();
        ac.request_join(id.public.clone());
        assert!(!ac.may_handshake(&id.public.node_id));
        ac.authorize(&id.public.node_id, "ops approve").unwrap();
        assert!(ac.may_handshake(&id.public.node_id));
    }

    #[test]
    fn revoked_blocks() {
        let id = NodeIdentity::generate("n".into(), "r".into(), Duration::hours(1));
        let mut ac = AdmissionController::new();
        ac.request_join(id.public.clone());
        ac.authorize(&id.public.node_id, "ok").unwrap();
        ac.revoke(&id.public.node_id, "compromised").unwrap();
        assert!(!ac.may_handshake(&id.public.node_id));
        assert!(ac.authorize(&id.public.node_id, "retry").is_err());
    }
}
