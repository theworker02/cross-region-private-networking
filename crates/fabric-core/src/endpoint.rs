//! Pluggable private endpoint provider (not AWS-hardcoded).

use crate::error::{FabricError, Result};
use crate::ids::{RegionID, ServiceID};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Capabilities a provider advertises.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EndpointCapabilities {
    /// Supports TLS to backend.
    pub tls: bool,
    /// Supports health probes.
    pub health: bool,
    /// Supports SNI override (with explicit cert policy).
    pub sni_override: bool,
    /// Provider id string.
    pub provider_id: String,
}

/// Discovered private endpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivateEndpoint {
    /// Logical alias (e.g. `database.internal`).
    pub alias: String,
    /// Provider-specific endpoint hostname (never treated as fabric identity).
    pub endpoint_host: String,
    /// Port.
    pub port: u16,
    /// Region hint.
    pub region: Option<RegionID>,
    /// Related service if any.
    pub service: Option<ServiceID>,
    /// Health.
    pub healthy: bool,
}

/// Certificate validation policy — hostname verification is NOT auto-disabled.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CertValidationPolicy {
    /// Verify cert against endpoint_host (default).
    VerifyEndpointHostname,
    /// Verify against an explicitly configured expected name (SNI override case).
    VerifyExpectedName {
        /// Expected DNS SAN / CN.
        expected_name: String,
    },
    /// Explicit insecure opt-in (must be named; never default).
    InsecureSkipVerifyExplicit {
        /// Operator justification recorded in receipts.
        justification: String,
    },
}

impl Default for CertValidationPolicy {
    fn default() -> Self {
        Self::VerifyEndpointHostname
    }
}

/// DNS alias → endpoint with cert policy.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DnsAlias {
    /// Alias name (`database.internal`).
    pub alias: String,
    /// Target endpoint.
    pub target: PrivateEndpoint,
    /// Optional SNI to send (does not imply skip-verify).
    pub sni: Option<String>,
    /// Cert validation policy.
    pub cert_policy: CertValidationPolicy,
}

/// Provider trait — discover/connect/health/resolve/capabilities.
pub trait PrivateEndpointProvider: Send + Sync {
    /// Provider name.
    fn id(&self) -> &str;
    /// Capabilities.
    fn capabilities(&self) -> EndpointCapabilities;
    /// Discover endpoints.
    fn discover(&self) -> Result<Vec<PrivateEndpoint>>;
    /// Resolve alias.
    fn resolve(&self, alias: &str) -> Result<PrivateEndpoint>;
    /// Health check.
    fn health(&self, endpoint: &PrivateEndpoint) -> Result<bool>;
    /// Connect metadata (logical — no raw sockets required for unit tests).
    fn connect_meta(&self, endpoint: &PrivateEndpoint) -> Result<String>;
}

/// In-memory provider for tests / local simulation.
#[derive(Default)]
pub struct MemoryEndpointProvider {
    endpoints: HashMap<String, PrivateEndpoint>,
}

impl MemoryEndpointProvider {
    /// Create empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register endpoint under alias.
    pub fn upsert(&mut self, ep: PrivateEndpoint) {
        self.endpoints.insert(ep.alias.clone(), ep);
    }
}

impl PrivateEndpointProvider for MemoryEndpointProvider {
    fn id(&self) -> &str {
        "memory"
    }

    fn capabilities(&self) -> EndpointCapabilities {
        EndpointCapabilities {
            tls: true,
            health: true,
            sni_override: true,
            provider_id: "memory".into(),
        }
    }

    fn discover(&self) -> Result<Vec<PrivateEndpoint>> {
        Ok(self.endpoints.values().cloned().collect())
    }

    fn resolve(&self, alias: &str) -> Result<PrivateEndpoint> {
        self.endpoints
            .get(alias)
            .cloned()
            .ok_or_else(|| FabricError::DnsFailed(format!("alias not found: {alias}")))
    }

    fn health(&self, endpoint: &PrivateEndpoint) -> Result<bool> {
        Ok(endpoint.healthy)
    }

    fn connect_meta(&self, endpoint: &PrivateEndpoint) -> Result<String> {
        if !endpoint.healthy {
            return Err(FabricError::NoRoute(format!(
                "endpoint {} unhealthy",
                endpoint.alias
            )));
        }
        Ok(format!(
            "connect {}:{} via provider={}",
            endpoint.endpoint_host,
            endpoint.port,
            self.id()
        ))
    }
}

/// Alias table with safe SNI / cert policy.
#[derive(Default)]
pub struct AliasTable {
    aliases: HashMap<String, DnsAlias>,
}

impl AliasTable {
    /// Create.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set alias. Rejects InsecureSkipVerify without justification text.
    pub fn set(&mut self, alias: DnsAlias) -> Result<()> {
        if let CertValidationPolicy::InsecureSkipVerifyExplicit { justification } =
            &alias.cert_policy
        {
            if justification.trim().is_empty() {
                return Err(FabricError::InvalidState(
                    "InsecureSkipVerify requires non-empty justification".into(),
                ));
            }
        }
        self.aliases.insert(alias.alias.clone(), alias);
        Ok(())
    }

    /// Get alias.
    pub fn get(&self, name: &str) -> Option<&DnsAlias> {
        self.aliases.get(name)
    }

    /// Resolve alias ensuring cert policy is explicit.
    pub fn resolve_safe(&self, name: &str) -> Result<&DnsAlias> {
        let a = self
            .get(name)
            .ok_or_else(|| FabricError::DnsFailed(format!("no alias: {name}")))?;
        // Never silently skip verify
        match &a.cert_policy {
            CertValidationPolicy::InsecureSkipVerifyExplicit { justification } => {
                tracing::warn!(
                    alias = %name,
                    justification = %justification,
                    "explicit insecure cert policy in use"
                );
            }
            _ => {}
        }
        Ok(a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_verifies_hostname() {
        assert!(matches!(
            CertValidationPolicy::default(),
            CertValidationPolicy::VerifyEndpointHostname
        ));
    }

    #[test]
    fn rejects_empty_insecure_justification() {
        let mut t = AliasTable::new();
        let err = t.set(DnsAlias {
            alias: "database.internal".into(),
            target: PrivateEndpoint {
                alias: "database.internal".into(),
                endpoint_host: "db.example.internal".into(),
                port: 5432,
                region: None,
                service: None,
                healthy: true,
            },
            sni: Some("db.example.internal".into()),
            cert_policy: CertValidationPolicy::InsecureSkipVerifyExplicit {
                justification: "".into(),
            },
        });
        assert!(err.is_err());
    }
}
