//! Global name resolution: `*.global.internal` — local native first, else fabric, else unavailable.

use crate::bypass::{classify_path, BypassDecision, PathClass};
use crate::dns::{FabricRoute, PrivateDns};
use crate::error::{FabricError, Result};
use crate::ids::{NodeID, RegionID, ServiceID};
use crate::ingress::IngressTransport;
use crate::registry::ServiceRegistry;
use crate::routing::RoutingEngine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Extended resolve answer with path classification.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalResolve {
    /// Name queried.
    pub name: String,
    /// Path class.
    pub path_class: PathClass,
    /// Native hostname when LOCAL_NATIVE.
    pub native_hostname: Option<String>,
    /// Fabric route when fabric path.
    pub fabric_route: Option<FabricRoute>,
    /// Reason.
    pub reason: String,
}

/// Map of service → same-region Render private hostname.
#[derive(Clone, Debug, Default)]
pub struct NativeHostnameMap {
    /// key: "service@region"
    map: HashMap<String, String>,
}

impl NativeHostnameMap {
    /// Create.
    pub fn new() -> Self {
        Self::default()
    }

    fn key(service: &ServiceID, region: &RegionID) -> String {
        format!("{}@{}", service, region)
    }

    /// Register Render private hostname for a service in a region.
    pub fn set(&mut self, service: &ServiceID, region: &RegionID, hostname: impl Into<String>) {
        self.map
            .insert(Self::key(service, region), hostname.into());
    }

    /// Lookup.
    pub fn get(&self, service: &ServiceID, region: &RegionID) -> Option<&str> {
        self.map.get(&Self::key(service, region)).map(|s| s.as_str())
    }
}

/// Parse `payments.global.internal` → service.
pub fn parse_global_name(name: &str) -> Result<ServiceID> {
    let lower = name.trim().to_ascii_lowercase();
    if let Some(svc) = lower.strip_suffix(".global.internal") {
        if svc.is_empty() || svc.contains('.') {
            return Err(FabricError::DnsFailed(format!(
                "bad global name: {name}"
            )));
        }
        return Ok(ServiceID::new(svc));
    }
    Err(FabricError::DnsFailed(format!(
        "not a .global.internal name: {name}"
    )))
}

/// Resolve global name with local-bypass preference.
pub fn resolve_global(
    name: &str,
    local_node: &NodeID,
    local_region: &RegionID,
    native: &NativeHostnameMap,
    registry: &ServiceRegistry,
    dns: &PrivateDns,
    router: &RoutingEngine,
    fabric_transport: IngressTransport,
) -> Result<GlobalResolve> {
    let service = parse_global_name(name)?;

    // 1) Local native if instance exists locally OR native hostname registered
    if let Some(host) = native.get(&service, local_region) {
        let local_inst = registry.lookup_in_region(&service, local_region);
        if !local_inst.is_empty() || !host.is_empty() {
            let d = classify_path(local_region, local_region, Some(host), None);
            return Ok(GlobalResolve {
                name: name.into(),
                path_class: d.class,
                native_hostname: d.native_hostname,
                fabric_route: None,
                reason: d.reason,
            });
        }
    }

    // Also: if registry has local healthy instance, require native hostname for LOCAL_NATIVE
    let local_inst = registry.lookup_in_region(&service, local_region);
    if let Some(inst) = local_inst.iter().find(|i| i.health.is_routable()) {
        if let Some(host) = native.get(&service, local_region) {
            return Ok(GlobalResolve {
                name: name.into(),
                path_class: PathClass::LocalNative,
                native_hostname: Some(host.into()),
                fabric_route: None,
                reason: format!(
                    "LOCAL_NATIVE instance {} via Render private hostname",
                    inst.node
                ),
            });
        }
    }

    // 2) Fabric path to remote
    let fabric_name = format!("{}.fabric.internal", service.as_str());
    match dns.resolve(&fabric_name, registry, router, local_node, local_region) {
        Ok(mut route) => {
            let bypass = classify_path(
                local_region,
                &route.target_region,
                None,
                Some(fabric_transport),
            );
            route.transport = crate::bypass::path_class_to_transport(bypass.class)
                .unwrap_or(route.transport);
            Ok(GlobalResolve {
                name: name.into(),
                path_class: bypass.class,
                native_hostname: None,
                fabric_route: Some(route),
                reason: bypass.reason,
            })
        }
        Err(_) => {
            // try legacy
            match dns.resolve(
                &format!("{}.internal", service.as_str()),
                registry,
                router,
                local_node,
                local_region,
            ) {
                Ok(mut route) => {
                    let bypass = classify_path(
                        local_region,
                        &route.target_region,
                        None,
                        Some(fabric_transport),
                    );
                    route.transport = crate::bypass::path_class_to_transport(bypass.class)
                        .unwrap_or(route.transport);
                    Ok(GlobalResolve {
                        name: name.into(),
                        path_class: bypass.class,
                        native_hostname: None,
                        fabric_route: Some(route),
                        reason: bypass.reason,
                    })
                }
                Err(e) => Ok(GlobalResolve {
                    name: name.into(),
                    path_class: PathClass::Unavailable,
                    native_hostname: None,
                    fabric_route: None,
                    reason: format!("UNAVAILABLE: {e}"),
                }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ServiceInstance;
    use chrono::Duration;

    #[test]
    fn global_prefers_local_native() {
        let reg = ServiceRegistry::new();
        reg.register(ServiceInstance::new(
            "payments", "n-va", "virginia", "prod", 8080,
        ));
        let mut native = NativeHostnameMap::new();
        native.set(&"payments".into(), &"virginia".into(), "payments-svc:8080");
        let dns = PrivateDns::new(Duration::seconds(30));
        let router = RoutingEngine::default();
        let r = resolve_global(
            "payments.global.internal",
            &"n-va".into(),
            &"virginia".into(),
            &native,
            &reg,
            &dns,
            &router,
            IngressTransport::PublicFabricIngress,
        )
        .unwrap();
        assert_eq!(r.path_class, PathClass::LocalNative);
    }

    #[test]
    fn global_cross_region_fabric() {
        let reg = ServiceRegistry::new();
        let mut fra = ServiceInstance::new("payments", "n-fra", "frankfurt", "prod", 8080);
        fra.latency_ms = 90.0;
        reg.register(fra);
        let native = NativeHostnameMap::new();
        let dns = PrivateDns::new(Duration::seconds(30));
        let router = RoutingEngine::default();
        let r = resolve_global(
            "payments.global.internal",
            &"n-va".into(),
            &"virginia".into(),
            &native,
            &reg,
            &dns,
            &router,
            IngressTransport::PublicFabricIngress,
        )
        .unwrap();
        assert_eq!(r.path_class, PathClass::FabricRelayed);
        assert!(r.fabric_route.unwrap().cross_region);
    }
}
