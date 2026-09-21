//! Membership view with version vectors — consistent-enough, no revoked resurrection.

use crate::ids::NodeID;
use crate::admission::AdmissionState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Membership entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemberView {
    /// Node.
    pub node: NodeID,
    /// Admission state.
    pub admission: AdmissionState,
    /// Policy version observed.
    pub policy_version: u64,
    /// Registry version observed.
    pub registry_version: u64,
    /// Revocation epoch (monotonic); higher wins.
    pub revocation_epoch: u64,
}

/// Lightweight membership gossip state.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MembershipConsensus {
    members: HashMap<String, MemberView>,
    /// Local logical clock.
    pub clock: u64,
}

impl MembershipConsensus {
    /// Create.
    pub fn new() -> Self {
        Self::default()
    }

    /// Upsert local observation.
    pub fn observe(&mut self, view: MemberView) {
        self.clock += 1;
        let key = view.node.to_string();
        match self.members.get(&key) {
            Some(existing) if existing.revocation_epoch > view.revocation_epoch => {
                // Ignore stale — never resurrect revoked with older epoch.
            }
            Some(existing)
                if existing.admission == AdmissionState::Revoked
                    && view.admission != AdmissionState::Revoked
                    && view.revocation_epoch <= existing.revocation_epoch =>
            {
                // Block revocation resurrection without bumping epoch.
            }
            _ => {
                self.members.insert(key, view);
            }
        }
    }

    /// Merge remote view (max epochs; revoked sticks unless higher epoch re-admit).
    pub fn merge(&mut self, remote: &MembershipConsensus) {
        for (k, v) in &remote.members {
            match self.members.get(k) {
                None => {
                    self.members.insert(k.clone(), v.clone());
                }
                Some(local) => {
                    if v.revocation_epoch > local.revocation_epoch {
                        self.members.insert(k.clone(), v.clone());
                    } else if v.revocation_epoch == local.revocation_epoch {
                        // Prefer Revoked over Active at same epoch.
                        if v.admission == AdmissionState::Revoked
                            || (local.admission != AdmissionState::Revoked
                                && v.policy_version >= local.policy_version)
                        {
                            if !(local.admission == AdmissionState::Revoked
                                && v.admission != AdmissionState::Revoked)
                            {
                                self.members.insert(k.clone(), v.clone());
                            }
                        }
                    }
                }
            }
        }
        self.clock = self.clock.max(remote.clock) + 1;
    }

    /// Lookup.
    pub fn get(&self, node: &NodeID) -> Option<&MemberView> {
        self.members.get(node.as_str())
    }

    /// All members.
    pub fn all(&self) -> Vec<&MemberView> {
        self.members.values().collect()
    }
}

/// Partition awareness status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PartitionStatus {
    /// Full connectivity to expected peers.
    Connected,
    /// Some links degraded.
    Degraded,
    /// Split into multiple components.
    Partitioned,
    /// No peers reachable.
    Isolated,
    /// Insufficient data.
    Unknown,
}

/// Assess partition from reachable peer count vs expected.
pub fn assess_partition(expected_peers: usize, reachable: usize, degraded_links: usize) -> PartitionStatus {
    if expected_peers == 0 {
        return PartitionStatus::Unknown;
    }
    if reachable == 0 {
        return PartitionStatus::Isolated;
    }
    if reachable < expected_peers {
        return PartitionStatus::Partitioned;
    }
    if degraded_links > 0 {
        return PartitionStatus::Degraded;
    }
    PartitionStatus::Connected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_stale_revoked_resurrection() {
        let mut a = MembershipConsensus::new();
        a.observe(MemberView {
            node: "n1".into(),
            admission: AdmissionState::Revoked,
            policy_version: 1,
            registry_version: 1,
            revocation_epoch: 5,
        });
        let mut b = MembershipConsensus::new();
        b.observe(MemberView {
            node: "n1".into(),
            admission: AdmissionState::Active,
            policy_version: 2,
            registry_version: 2,
            revocation_epoch: 4, // older
        });
        a.merge(&b);
        assert_eq!(
            a.get(&"n1".into()).unwrap().admission,
            AdmissionState::Revoked
        );
    }
}
