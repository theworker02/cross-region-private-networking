//! Distributed service registry with local store + gossip/sync interface.

use crate::ids::{NodeID, RegionID, ServiceID};
use crate::types::{HealthState, ServiceInstance};
use chrono::Utc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Gossip / sync delta for registry replication between peers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegistryDelta {
    /// Instances to upsert.
    pub upserts: Vec<ServiceInstance>,
    /// Instance keys to remove (`service@node`).
    pub deletes: Vec<String>,
    /// Monotonic sync version from sender.
    pub version: u64,
    /// Sender node.
    pub from: NodeID,
}

fn instance_key(svc: &ServiceID, node: &NodeID) -> String {
    format!("{}@{}", svc.as_str(), node.as_str())
}

/// Local registry shard with sync interface for mesh gossip.
#[derive(Clone, Default)]
pub struct ServiceRegistry {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Default)]
struct Inner {
    instances: HashMap<String, ServiceInstance>,
    version: u64,
}

impl ServiceRegistry {
    /// Empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register or refresh an instance. Returns new local version.
    pub fn register(&self, instance: ServiceInstance) -> u64 {
        let mut g = self.inner.write();
        g.version += 1;
        let key = instance_key(&instance.service, &instance.node);
        g.instances.insert(key, instance);
        g.version
    }

    /// Deregister a specific instance.
    pub fn deregister(&self, service: &ServiceID, node: &NodeID) -> bool {
        let mut g = self.inner.write();
        let key = instance_key(service, node);
        if g.instances.remove(&key).is_some() {
            g.version += 1;
            true
        } else {
            false
        }
    }

    /// Deregister all instances for a node (crash / leave).
    pub fn deregister_node(&self, node: &NodeID) -> usize {
        let mut g = self.inner.write();
        let before = g.instances.len();
        g.instances.retain(|_, i| &i.node != node);
        let removed = before - g.instances.len();
        if removed > 0 {
            g.version += 1;
        }
        removed
    }

    /// Update health for an instance.
    pub fn update_health(&self, service: &ServiceID, node: &NodeID, health: HealthState) -> bool {
        let mut g = self.inner.write();
        let key = instance_key(service, node);
        if let Some(inst) = g.instances.get_mut(&key) {
            inst.health = health;
            inst.last_seen = Utc::now();
            g.version += 1;
            true
        } else {
            false
        }
    }

    /// Update latency/loss observations.
    pub fn update_metrics(
        &self,
        service: &ServiceID,
        node: &NodeID,
        latency_ms: f64,
        loss: f64,
    ) -> bool {
        let mut g = self.inner.write();
        let key = instance_key(service, node);
        if let Some(inst) = g.instances.get_mut(&key) {
            inst.latency_ms = latency_ms;
            inst.loss = loss;
            inst.last_seen = Utc::now();
            true
        } else {
            false
        }
    }

    /// All instances of a service (any region).
    pub fn lookup(&self, service: &ServiceID) -> Vec<ServiceInstance> {
        let g = self.inner.read();
        g.instances
            .values()
            .filter(|i| &i.service == service)
            .cloned()
            .collect()
    }

    /// Instances in a specific region.
    pub fn lookup_in_region(&self, service: &ServiceID, region: &RegionID) -> Vec<ServiceInstance> {
        self.lookup(service)
            .into_iter()
            .filter(|i| &i.region == region)
            .collect()
    }

    /// Instance on a specific node.
    pub fn lookup_on_node(&self, service: &ServiceID, node: &NodeID) -> Option<ServiceInstance> {
        let g = self.inner.read();
        g.instances.get(&instance_key(service, node)).cloned()
    }

    /// Snapshot all instances.
    pub fn all(&self) -> Vec<ServiceInstance> {
        self.inner.read().instances.values().cloned().collect()
    }

    /// Current sync version.
    pub fn version(&self) -> u64 {
        self.inner.read().version
    }

    /// Export a delta for gossip to peers.
    pub fn export_delta(&self, from: &NodeID) -> RegistryDelta {
        let g = self.inner.read();
        RegistryDelta {
            upserts: g.instances.values().cloned().collect(),
            deletes: Vec::new(),
            version: g.version,
            from: from.clone(),
        }
    }

    /// Apply a remote gossip delta (last-write by version, keyed merge).
    pub fn apply_delta(&self, delta: RegistryDelta) -> u64 {
        let mut g = self.inner.write();
        for key in delta.deletes {
            g.instances.remove(&key);
        }
        for inst in delta.upserts {
            let key = instance_key(&inst.service, &inst.node);
            match g.instances.get(&key) {
                Some(existing) if existing.last_seen > inst.last_seen => {}
                _ => {
                    g.instances.insert(key, inst);
                }
            }
        }
        g.version = g.version.max(delta.version) + 1;
        g.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ServiceInstance;

    #[test]
    fn register_lookup_deregister() {
        let reg = ServiceRegistry::new();
        let inst = ServiceInstance::new("payments", "node-17", "eu-central", "prod", 8080);
        reg.register(inst);
        assert_eq!(reg.lookup(&"payments".into()).len(), 1);
        assert!(reg.deregister(&"payments".into(), &"node-17".into()));
        assert!(reg.lookup(&"payments".into()).is_empty());
    }

    #[test]
    fn gossip_sync() {
        let a = ServiceRegistry::new();
        let b = ServiceRegistry::new();
        a.register(ServiceInstance::new(
            "payments",
            "fra-1",
            "eu-central",
            "prod",
            8080,
        ));
        let delta = a.export_delta(&"va-1".into());
        b.apply_delta(delta);
        assert_eq!(b.lookup(&"payments".into()).len(), 1);
    }
}
