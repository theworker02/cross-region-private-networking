//! Policy version distribution with digests, epochs, and ack.

use crate::error::{FabricError, Result};
use crate::policy::PolicyRuleDecl;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Local policy freshness relative to the mesh.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PolicyFreshness {
    /// Matches current epoch/digest.
    Current,
    /// Older but still structurally valid.
    Stale,
    /// Digest/epoch mismatch that must not be applied.
    Invalid,
    /// No policy known.
    Unknown,
}

/// Distributed policy bundle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyBundle {
    /// Monotonic version.
    pub version: u64,
    /// Epoch (rotation boundary).
    pub epoch: u64,
    /// Hex SHA-256 of canonical JSON rules.
    pub digest: String,
    /// Rules.
    pub rules: Vec<PolicyRuleDecl>,
}

impl PolicyBundle {
    /// Compile bundle from rules.
    pub fn from_rules(version: u64, epoch: u64, rules: Vec<PolicyRuleDecl>) -> Result<Self> {
        let canonical = serde_json::to_string(&rules)?;
        let mut h = Sha256::new();
        h.update(canonical.as_bytes());
        let digest = hex::encode(h.finalize());
        Ok(Self {
            version,
            epoch,
            digest,
            rules,
        })
    }

    /// Verify digest matches rules.
    pub fn verify_digest(&self) -> Result<()> {
        let canonical = serde_json::to_string(&self.rules)?;
        let mut h = Sha256::new();
        h.update(canonical.as_bytes());
        let d = hex::encode(h.finalize());
        if d != self.digest {
            return Err(FabricError::InvalidState(
                "policy digest mismatch — refusing to activate INVALID policy".into(),
            ));
        }
        Ok(())
    }
}

/// Per-node ack of a policy version.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyAck {
    /// Node id string.
    pub node: String,
    /// Version acked.
    pub version: u64,
    /// Epoch acked.
    pub epoch: u64,
    /// Digest acked.
    pub digest: String,
}

/// Distributor / tracker.
#[derive(Clone, Debug, Default)]
pub struct PolicyDistributor {
    current: Option<PolicyBundle>,
    acks: Vec<PolicyAck>,
}

impl PolicyDistributor {
    /// Create.
    pub fn new() -> Self {
        Self::default()
    }

    /// Activate only if digest verifies.
    pub fn activate(&mut self, bundle: PolicyBundle) -> Result<()> {
        bundle.verify_digest()?;
        self.current = Some(bundle);
        Ok(())
    }

    /// Record ack.
    pub fn ack(&mut self, ack: PolicyAck) {
        self.acks.push(ack);
    }

    /// Freshness of a peer's claimed version/digest vs current.
    pub fn freshness(&self, version: u64, epoch: u64, digest: &str) -> PolicyFreshness {
        let Some(cur) = &self.current else {
            return PolicyFreshness::Unknown;
        };
        if digest == cur.digest && version == cur.version && epoch == cur.epoch {
            return PolicyFreshness::Current;
        }
        if epoch > cur.epoch {
            // Future epoch with wrong digest → invalid
            if digest != cur.digest && version != cur.version {
                return PolicyFreshness::Invalid;
            }
        }
        if digest != cur.digest && version > cur.version {
            // Claimed newer version but digest doesn't match any known → invalid
            return PolicyFreshness::Invalid;
        }
        if version < cur.version || epoch < cur.epoch {
            return PolicyFreshness::Stale;
        }
        if digest != cur.digest {
            return PolicyFreshness::Invalid;
        }
        PolicyFreshness::Stale
    }

    /// Current bundle.
    pub fn current(&self) -> Option<&PolicyBundle> {
        self.current.as_ref()
    }

    /// Never silently run invalid — helper for callers.
    pub fn require_current_or_stale(&self, freshness: PolicyFreshness) -> Result<()> {
        match freshness {
            PolicyFreshness::Current | PolicyFreshness::Stale => Ok(()),
            PolicyFreshness::Invalid => Err(FabricError::InvalidState(
                "INVALID policy — refusing to enforce".into(),
            )),
            PolicyFreshness::Unknown => Err(FabricError::InvalidState(
                "UNKNOWN policy — refusing to enforce".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::parse_policy_line;

    #[test]
    fn rejects_tampered_digest() {
        let rules = vec![parse_policy_line(
            "allow from checkout to payments port 8080 proto http priority 10",
        )
        .unwrap()];
        let mut b = PolicyBundle::from_rules(1, 1, rules).unwrap();
        b.digest = "deadbeef".into();
        let mut d = PolicyDistributor::new();
        assert!(d.activate(b).is_err());
    }

    #[test]
    fn freshness_states() {
        let rules = vec![parse_policy_line(
            "allow from checkout to payments port 8080 proto http priority 10",
        )
        .unwrap()];
        let b = PolicyBundle::from_rules(2, 1, rules).unwrap();
        let digest = b.digest.clone();
        let mut d = PolicyDistributor::new();
        d.activate(b).unwrap();
        assert_eq!(d.freshness(2, 1, &digest), PolicyFreshness::Current);
        assert_eq!(d.freshness(1, 1, &digest), PolicyFreshness::Stale);
        assert_eq!(d.freshness(3, 1, "ff00"), PolicyFreshness::Invalid);
    }
}
