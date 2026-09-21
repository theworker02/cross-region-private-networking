//! Cross-region private hostname resolution.
//!
//! Resolves fabric names to **fabric routes** (node / region / transport), never
//! public application endpoints. Supports unqualified, region-qualified, and
//! instance-qualified names with TTL cache invalidated on health failure or
//! deregistration.

use crate::error::{FabricError, Result};
use crate::ids::{NodeID, RegionID, ServiceID};
use crate::namespace::FabricName;
use crate::registry::ServiceRegistry;
use crate::routing::{RouteDecision, RoutingEngine};
use crate::types::{HealthState, TransportMode};
use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Parsed form of a fabric hostname.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostnameQuery {
    /// `payments.internal` — best eligible instance (may be remote).
    Service(ServiceID),
    /// `payments.us-east.internal` — prefer/require that region.
    ServiceRegion {
        /// Service short name.
        service: ServiceID,
        /// Region qualifier.
        region: RegionID,
    },
    /// `payments.node-17.internal` — specific instance.
    ServiceNode {
        /// Service short name.
        service: ServiceID,
        /// Node qualifier.
        node: NodeID,
    },
}

impl HostnameQuery {
    /// Parse `*.fabric.internal` or legacy `*.internal`.
    pub fn parse(name: &str) -> Option<Self> {
        let fab = FabricName::parse(name).ok()?;
        match (fab.region, fab.node) {
            (None, None) => Some(HostnameQuery::Service(fab.service)),
            (Some(region), None) => Some(HostnameQuery::ServiceRegion {
                service: fab.service,
                region,
            }),
            (None, Some(node)) => Some(HostnameQuery::ServiceNode {
                service: fab.service,
                node,
            }),
            (Some(region), Some(_)) => Some(HostnameQuery::ServiceRegion {
                service: fab.service,
                region,
            }),
        }
    }
}

/// A fabric route answer — never a public app URL.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FabricRoute {
    /// Original query name.
    pub name: String,
    /// Resolved service.
    pub service: ServiceID,
    /// Target fabric node that hosts / fronts the instance.
    pub target_node: NodeID,
    /// Region of the target instance.
    pub target_region: RegionID,
    /// Local fabric node that will forward (caller's region gateway).
    pub via_local_node: NodeID,
    /// Caller's region.
    pub local_region: RegionID,
    /// Whether the instance is in a different region.
    pub cross_region: bool,
    /// How the path is transported.
    pub transport: TransportMode,
    /// Instance listen port (private fabric metadata).
    pub port: u16,
    /// Protocol hint.
    pub protocol: String,
    /// Health at resolve time.
    pub health: HealthState,
    /// Routing explanation summary.
    pub reason: String,
    /// Observed or estimated latency hint (ms).
    pub latency_ms: f64,
}

/// Cached DNS answer with TTL.
#[derive(Clone, Debug)]
struct CacheEntry {
    route: FabricRoute,
    expires_at: DateTime<Utc>,
    /// Registry version at insert — used to detect deregistration.
    registry_version: u64,
    /// Target node at insert — invalidate if health goes bad.
    target_node: NodeID,
    service: ServiceID,
}

/// Private DNS resolver with cache + stale-route prevention.
pub struct PrivateDns {
    cache: RwLock<HashMap<String, CacheEntry>>,
    /// Default TTL for positive answers.
    pub ttl: Duration,
    /// Negative cache TTL (brief).
    pub negative_ttl: Duration,
}

impl Default for PrivateDns {
    fn default() -> Self {
        Self::new(Duration::seconds(30))
    }
}

impl PrivateDns {
    /// Create resolver with positive TTL.
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            ttl,
            negative_ttl: Duration::seconds(2),
        }
    }

    /// Invalidate entire cache.
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }

    /// Invalidate entries for a service (any qualifier).
    pub fn invalidate_service(&self, service: &ServiceID) {
        let mut g = self.cache.write();
        g.retain(|_, e| &e.service != service);
    }

    /// Invalidate entries targeting a node.
    pub fn invalidate_node(&self, node: &NodeID) {
        let mut g = self.cache.write();
        g.retain(|_, e| &e.target_node != node);
    }

    /// Invalidate a specific name.
    pub fn invalidate_name(&self, name: &str) {
        self.cache.write().remove(&name.to_ascii_lowercase());
    }

    /// Called when health fails — drop stale routes to that instance.
    pub fn on_health_failure(&self, service: &ServiceID, node: &NodeID) {
        let mut g = self.cache.write();
        g.retain(|_, e| !(&e.service == service && &e.target_node == node));
    }

    /// Resolve hostname to a fabric route using registry + routing engine.
    pub fn resolve(
        &self,
        name: &str,
        registry: &ServiceRegistry,
        router: &RoutingEngine,
        local_node: &NodeID,
        local_region: &RegionID,
    ) -> Result<FabricRoute> {
        let key = name.to_ascii_lowercase();
        let query = HostnameQuery::parse(&key)
            .ok_or_else(|| FabricError::DnsFailed(format!("not a fabric name: {name}")))?;

        // Serve cache only if still fresh AND target still healthy in registry.
        if let Some(hit) = self.try_cache_hit(&key, registry)? {
            return Ok(hit);
        }

        let decision = self.select(&query, registry, router, local_region)?;
        let inst = decision.instance;
        let cross = &inst.region != local_region;
        let route = FabricRoute {
            name: key.clone(),
            service: inst.service.clone(),
            target_node: inst.node.clone(),
            target_region: inst.region.clone(),
            via_local_node: local_node.clone(),
            local_region: local_region.clone(),
            cross_region: cross,
            transport: decision.transport,
            port: inst.port,
            protocol: inst.protocol.clone(),
            health: inst.health,
            reason: decision.reason,
            latency_ms: decision.score_latency_ms,
        };

        self.cache.write().insert(
            key,
            CacheEntry {
                route: route.clone(),
                expires_at: Utc::now() + self.ttl,
                registry_version: registry.version(),
                target_node: route.target_node.clone(),
                service: route.service.clone(),
            },
        );
        Ok(route)
    }

    fn try_cache_hit(&self, key: &str, registry: &ServiceRegistry) -> Result<Option<FabricRoute>> {
        let mut g = self.cache.write();
        let Some(entry) = g.get(key) else {
            return Ok(None);
        };
        if Utc::now() >= entry.expires_at {
            g.remove(key);
            return Ok(None);
        }
        // Stale prevention: registry changed or target unhealthy/missing.
        if registry.version() != entry.registry_version {
            // Soft check — verify target still exists and is routable.
        }
        match registry.lookup_on_node(&entry.service, &entry.target_node) {
            Some(inst) if inst.health.is_routable() => {
                // Refresh route health/latency from live instance.
                let mut route = entry.route.clone();
                route.health = inst.health;
                route.latency_ms = inst.latency_ms;
                Ok(Some(route))
            }
            _ => {
                // Dead / deregistered — do not silently route.
                g.remove(key);
                Ok(None)
            }
        }
    }

    fn select(
        &self,
        query: &HostnameQuery,
        registry: &ServiceRegistry,
        router: &RoutingEngine,
        local_region: &RegionID,
    ) -> Result<RouteDecision> {
        match query {
            HostnameQuery::Service(svc) => {
                let candidates = registry.lookup(svc);
                router.choose(svc, &candidates, local_region, None)
            }
            HostnameQuery::ServiceRegion { service, region } => {
                let candidates = registry.lookup_in_region(service, region);
                if candidates.is_empty() {
                    return Err(FabricError::DnsFailed(format!(
                        "no instances of {} in region {}",
                        service, region
                    )));
                }
                router.choose(service, &candidates, local_region, Some(region))
            }
            HostnameQuery::ServiceNode { service, node } => {
                let Some(inst) = registry.lookup_on_node(service, node) else {
                    return Err(FabricError::DnsFailed(format!(
                        "no instance {} on {}",
                        service, node
                    )));
                };
                if !inst.health.is_routable() {
                    return Err(FabricError::DnsFailed(format!(
                        "instance {} on {} is {:?}",
                        service, node, inst.health
                    )));
                }
                Ok(RouteDecision {
                    instance: inst.clone(),
                    transport: TransportMode::Direct, // refined by fabric core with peer state
                    score: 0.0,
                    score_latency_ms: inst.latency_ms,
                    reason: format!("instance-qualified hostname → {}", node),
                    alternatives: vec![],
                })
            }
        }
    }
}

/// Shared DNS handle.
pub type SharedDns = Arc<PrivateDns>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::RoutingEngine;
    use crate::types::ServiceInstance;

    fn setup() -> (ServiceRegistry, RoutingEngine, PrivateDns) {
        let reg = ServiceRegistry::new();
        // Frankfurt-only payments — VA must resolve cross-region.
        let mut fra = ServiceInstance::new("payments", "node-17", "frankfurt", "prod", 8080);
        fra.latency_ms = 85.0;
        reg.register(fra);
        let mut east = ServiceInstance::new("checkout", "node-va", "virginia", "prod", 9000);
        east.latency_ms = 2.0;
        reg.register(east);
        (reg, RoutingEngine::default(), PrivateDns::new(Duration::seconds(60)))
    }

    #[test]
    fn parse_hostname_forms() {
        assert_eq!(
            HostnameQuery::parse("payments.internal"),
            Some(HostnameQuery::Service("payments".into()))
        );
        assert_eq!(
            HostnameQuery::parse("payments.us-east.internal"),
            Some(HostnameQuery::ServiceRegion {
                service: "payments".into(),
                region: "us-east".into(),
            })
        );
        assert_eq!(
            HostnameQuery::parse("payments.node-17.internal"),
            Some(HostnameQuery::ServiceNode {
                service: "payments".into(),
                node: "node-17".into(),
            })
        );
        assert!(HostnameQuery::parse("payments.com").is_none());
    }

    #[test]
    fn va_resolves_frankfurt_only_service() {
        let (reg, router, dns) = setup();
        let route = dns
            .resolve(
                "payments.internal",
                &reg,
                &router,
                &"node-va".into(),
                &"virginia".into(),
            )
            .unwrap();
        assert!(route.cross_region);
        assert_eq!(route.target_region.as_str(), "frankfurt");
        assert_eq!(route.target_node.as_str(), "node-17");
        assert_eq!(route.port, 8080);
        // Fabric route — not a public hostname.
        assert_eq!(route.via_local_node.as_str(), "node-va");
    }

    #[test]
    fn region_qualified_vs_unqualified() {
        let (reg, router, dns) = setup();
        let mut us_pay = ServiceInstance::new("payments", "node-va2", "virginia", "prod", 8081);
        us_pay.latency_ms = 3.0;
        reg.register(us_pay);

        let unqual = dns
            .resolve(
                "payments.internal",
                &reg,
                &router,
                &"node-va".into(),
                &"virginia".into(),
            )
            .unwrap();
        // Unqualified prefers local region when healthy.
        assert_eq!(unqual.target_region.as_str(), "virginia");

        let remote = dns
            .resolve(
                "payments.frankfurt.internal",
                &reg,
                &router,
                &"node-va".into(),
                &"virginia".into(),
            )
            .unwrap();
        assert_eq!(remote.target_region.as_str(), "frankfurt");
        assert!(remote.cross_region);
    }

    #[test]
    fn stale_remote_invalidated_on_health_failure() {
        let (reg, router, dns) = setup();
        let r1 = dns
            .resolve(
                "payments.internal",
                &reg,
                &router,
                &"node-va".into(),
                &"virginia".into(),
            )
            .unwrap();
        assert_eq!(r1.target_node.as_str(), "node-17");

        reg.update_health(
            &"payments".into(),
            &"node-17".into(),
            HealthState::Unreachable,
        );
        dns.on_health_failure(&"payments".into(), &"node-17".into());

        let err = dns
            .resolve(
                "payments.internal",
                &reg,
                &router,
                &"node-va".into(),
                &"virginia".into(),
            )
            .unwrap_err();
        assert!(matches!(err, FabricError::NoRoute(_) | FabricError::DnsFailed(_)));
    }

    #[test]
    fn instance_qualified() {
        let (reg, router, dns) = setup();
        let r = dns
            .resolve(
                "payments.node-17.internal",
                &reg,
                &router,
                &"node-va".into(),
                &"virginia".into(),
            )
            .unwrap();
        assert_eq!(r.target_node.as_str(), "node-17");
    }

    #[test]
    fn fabric_internal_namespace() {
        let (reg, router, dns) = setup();
        let r = dns
            .resolve(
                "payments.fabric.internal",
                &reg,
                &router,
                &"node-va".into(),
                &"virginia".into(),
            )
            .unwrap();
        assert!(r.cross_region);
        let r2 = dns
            .resolve(
                "payments.frankfurt.fabric.internal",
                &reg,
                &router,
                &"node-va".into(),
                &"virginia".into(),
            )
            .unwrap();
        assert_eq!(r2.target_region.as_str(), "frankfurt");
    }
}
