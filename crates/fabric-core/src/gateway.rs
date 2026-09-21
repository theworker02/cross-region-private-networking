//! Bidirectional gateway mode — OPTIONAL, explicit authorization only.

use crate::error::{FabricError, Result};
use crate::ids::{NodeID, ServiceID};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Gateway direction authorization.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GatewayAuth {
    /// Allowed inbound service identities (empty = none).
    pub allow_inbound_services: HashSet<String>,
    /// Allowed outbound service identities.
    pub allow_outbound_services: HashSet<String>,
    /// Allowed peer nodes for bidirectional bridging.
    pub allow_peer_nodes: HashSet<String>,
    /// Master switch — must be explicitly enabled.
    pub bidirectional_enabled: bool,
}

impl GatewayAuth {
    /// Disabled by default (no public unauthenticated ingress).
    pub fn disabled() -> Self {
        Self::default()
    }

    /// Enable bidirectional with explicit peer + service allowlists.
    pub fn enable_bidirectional(
        &mut self,
        peers: impl IntoIterator<Item = NodeID>,
        inbound: impl IntoIterator<Item = ServiceID>,
        outbound: impl IntoIterator<Item = ServiceID>,
    ) {
        self.bidirectional_enabled = true;
        self.allow_peer_nodes = peers.into_iter().map(|n| n.to_string()).collect();
        self.allow_inbound_services = inbound.into_iter().map(|s| s.to_string()).collect();
        self.allow_outbound_services = outbound.into_iter().map(|s| s.to_string()).collect();
    }

    /// Authorize inbound (requires enabled + allowlists).
    pub fn authorize_inbound(&self, peer: &NodeID, service: &ServiceID) -> Result<()> {
        if !self.bidirectional_enabled {
            return Err(FabricError::AuthFailed(
                "bidirectional gateway disabled (default)".into(),
            ));
        }
        if !self.allow_peer_nodes.contains(peer.as_str()) {
            return Err(FabricError::AuthFailed(format!(
                "peer {peer} not authorized for gateway"
            )));
        }
        if !self.allow_inbound_services.contains(service.as_str()) {
            return Err(FabricError::PolicyDenied(format!(
                "inbound service {service} not on gateway allowlist"
            )));
        }
        Ok(())
    }
}

/// Cross-region private service bridge descriptor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceBridge {
    /// Local region fabric node.
    pub local_node: NodeID,
    /// Remote region fabric node.
    pub remote_node: NodeID,
    /// Exported remote services available locally via fabric namespace.
    pub imported_services: Vec<ServiceID>,
    /// Honest note: peer link may traverse public Internet between fabric nodes.
    pub transit_note: String,
}

impl ServiceBridge {
    /// Create VA↔FRA style bridge metadata.
    pub fn new(local: NodeID, remote: NodeID, services: Vec<ServiceID>) -> Self {
        Self {
            local_node: local,
            remote_node: remote,
            imported_services: services,
            transit_note: "Fabric nodes authenticate over encrypted transit that may use \
                public infrastructure (e.g. public fabric ingress with mTLS). Application \
                workloads keep private semantics via fabric names; this is NOT Render-native \
                cross-region private networking."
                .into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_rejects_inbound() {
        let g = GatewayAuth::disabled();
        assert!(g
            .authorize_inbound(&"n1".into(), &"api".into())
            .is_err());
    }

    #[test]
    fn explicit_enable_works() {
        let mut g = GatewayAuth::disabled();
        g.enable_bidirectional(
            vec!["n1".into()],
            vec!["api".into()],
            vec!["api".into()],
        );
        assert!(g.authorize_inbound(&"n1".into(), &"api".into()).is_ok());
    }
}
