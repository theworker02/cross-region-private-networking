//! Relay fallback and NAT/restricted connectivity helpers.

use crate::error::{FabricError, Result};
use crate::ids::NodeID;
use crate::ingress::IngressTransport;
use crate::types::TransportMode;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Authorized relay registry.
#[derive(Clone, Debug, Default)]
pub struct RelayRegistry {
    authorized: HashSet<String>,
}

impl RelayRegistry {
    /// Empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// Authorize a relay node.
    pub fn authorize(&mut self, node: &NodeID) {
        self.authorized.insert(node.to_string());
    }

    /// Revoke relay authorization.
    pub fn deauthorize(&mut self, node: &NodeID) {
        self.authorized.remove(node.as_str());
    }

    /// Is node an authorized relay?
    pub fn is_authorized(&self, node: &NodeID) -> bool {
        self.authorized.contains(node.as_str())
    }
}

/// Outcome of path selection under NAT / restricted connectivity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectivityPath {
    /// Selected transport.
    pub transport: IngressTransport,
    /// Mapped dataplane mode (RELAYED for ingress/relay — never labeled DIRECT).
    pub mode: TransportMode,
    /// Relay or ingress peer if any.
    pub via: Option<NodeID>,
    /// Explanation.
    pub reason: String,
}

/// Try DIRECT, then public fabric ingress, then authorized relay.
pub fn resolve_restricted_path(
    direct_reachable: bool,
    ingress_available: bool,
    preferred_relay: Option<&NodeID>,
    relays: &RelayRegistry,
) -> Result<ConnectivityPath> {
    if direct_reachable {
        return Ok(ConnectivityPath {
            transport: IngressTransport::Direct,
            mode: TransportMode::Direct,
            via: None,
            reason: "authenticated DIRECT path available".into(),
        });
    }
    if ingress_available {
        return Ok(ConnectivityPath {
            transport: IngressTransport::PublicFabricIngress,
            mode: TransportMode::Relayed,
            via: None,
            reason: "DIRECT unavailable; using PUBLIC_FABRIC_INGRESS (mTLS+allowlist over Internet — not a private network)".into(),
        });
    }
    if let Some(r) = preferred_relay {
        if relays.is_authorized(r) {
            return Ok(ConnectivityPath {
                transport: IngressTransport::AuthorizedRelay,
                mode: TransportMode::Relayed,
                via: Some(r.clone()),
                reason: format!("DIRECT and ingress unavailable; AUTHORIZED_RELAY via {r}"),
            });
        }
        return Err(FabricError::RelayUnavailable(format!(
            "relay {r} not authorized"
        )));
    }
    Err(FabricError::RelayUnavailable(
        "no DIRECT, ingress, or authorized relay".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_labels_relay_as_direct() {
        let mut reg = RelayRegistry::new();
        reg.authorize(&"relay-1".into());
        let p = resolve_restricted_path(false, false, Some(&"relay-1".into()), &reg).unwrap();
        assert_eq!(p.mode, TransportMode::Relayed);
        assert_ne!(p.transport.label(), "DIRECT");
    }

    #[test]
    fn ingress_before_relay() {
        let reg = RelayRegistry::new();
        let p = resolve_restricted_path(false, true, None, &reg).unwrap();
        assert_eq!(p.transport, IngressTransport::PublicFabricIngress);
        assert_eq!(p.mode, TransportMode::Relayed);
    }
}
