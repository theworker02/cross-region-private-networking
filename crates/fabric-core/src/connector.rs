//! External network connector trait + local/test reference implementation.

use crate::error::{FabricError, Result};
use serde::{Deserialize, Serialize};

/// Capabilities advertised by an external connector (e.g. PrivateLink complement).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConnectorCapabilities {
    /// Connector id.
    pub id: String,
    /// Can discover endpoints.
    pub discover: bool,
    /// Can health-check.
    pub health: bool,
    /// TLS to resource.
    pub tls: bool,
    /// Documented only — no inventing vendor APIs.
    pub notes: String,
}

/// Resolved external resource handle (mock / local).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalResource {
    /// Logical name.
    pub name: String,
    /// Endpoint hostname (may be PrivateLink DNS name from dashboard — documented).
    pub endpoint: String,
    /// Port.
    pub port: u16,
    /// Healthy?
    pub healthy: bool,
}

/// Pluggable connector — not AWS-SDK-hardcoded.
pub trait ExternalNetworkConnector: Send + Sync {
    /// Id.
    fn id(&self) -> &str;
    /// Caps.
    fn capabilities(&self) -> ConnectorCapabilities;
    /// Discover.
    fn discover(&self) -> Result<Vec<ExternalResource>>;
    /// Connect metadata (logical for tests).
    fn connect(&self, name: &str) -> Result<String>;
    /// Health.
    fn health(&self, name: &str) -> Result<bool>;
}

/// In-process mock PrivateLink-complement resource (local/test reference).
#[derive(Clone, Debug, Default)]
pub struct LocalMockConnector {
    resources: Vec<ExternalResource>,
}

impl LocalMockConnector {
    /// Empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed a mock AWS-side resource reachable only via gateway region.
    pub fn with_mock_resource(mut self, name: &str, endpoint: &str, port: u16) -> Self {
        self.resources.push(ExternalResource {
            name: name.into(),
            endpoint: endpoint.into(),
            port,
            healthy: true,
        });
        self
    }
}

impl ExternalNetworkConnector for LocalMockConnector {
    fn id(&self) -> &str {
        "local-mock-privatelink-complement"
    }

    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            id: self.id().into(),
            discover: true,
            health: true,
            tls: true,
            notes: "Test double for PrivateLink complement mode. Real PL uses Render-documented DNS names; Private DNS unsupported per Render docs — use SNI override with explicit cert policy.".into(),
        }
    }

    fn discover(&self) -> Result<Vec<ExternalResource>> {
        Ok(self.resources.clone())
    }

    fn connect(&self, name: &str) -> Result<String> {
        let r = self
            .resources
            .iter()
            .find(|r| r.name == name)
            .ok_or_else(|| FabricError::NoRoute(format!("external resource not found: {name}")))?;
        if !r.healthy {
            return Err(FabricError::NoRoute(format!("{name} unhealthy")));
        }
        Ok(format!(
            "connect tls://{}:{} via connector={} (complement path A→fabric→B→gateway→resource)",
            r.endpoint,
            r.port,
            self.id()
        ))
    }

    fn health(&self, name: &str) -> Result<bool> {
        Ok(self
            .resources
            .iter()
            .find(|r| r.name == name)
            .map(|r| r.healthy)
            .unwrap_or(false))
    }
}

/// PrivateLink complement path description (documented capabilities only).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivateLinkComplementPlan {
    /// Caller region.
    pub from_region: String,
    /// Region that holds the Render PrivateLink attachment.
    pub gateway_region: String,
    /// External resource name.
    pub resource: String,
    /// Honest underlay note.
    pub underlay_note: String,
}

impl PrivateLinkComplementPlan {
    /// Build plan: remote → fabric → PL region → AWS resource.
    pub fn new(from_region: &str, gateway_region: &str, resource: &str) -> Self {
        Self {
            from_region: from_region.into(),
            gateway_region: gateway_region.into(),
            resource: resource.into(),
            underlay_note: "Complement mode: cross-region fabric reaches the Render region that \
                owns the PrivateLink; PrivateLink itself follows Render public docs (Render→AWS, \
                not reverse; Private DNS unsupported). Fabric underlay between regions is not \
                Render private networking."
                .into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_connector_connects() {
        let c = LocalMockConnector::new().with_mock_resource(
            "aurora",
            "vpce-docs-example.amazonaws.com",
            5432,
        );
        assert!(c.health("aurora").unwrap());
        assert!(c.connect("aurora").unwrap().contains("fabric"));
    }
}
