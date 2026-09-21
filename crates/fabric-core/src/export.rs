//! Service export/import — default unexported.

use crate::error::{FabricError, Result};
use crate::ids::{NetworkID, NodeID, RegionID, ServiceID};
use crate::types::ServiceInstance;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Export record for a service.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceExport {
    /// Service.
    pub service: ServiceID,
    /// Exporting node.
    pub from_node: NodeID,
    /// Exporting region.
    pub from_region: RegionID,
    /// Network.
    pub network: NetworkID,
    /// Instance snapshot.
    pub instance: ServiceInstance,
    /// Explicit export flag (must be true to share).
    pub exported: bool,
}

/// Import acceptance.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceImport {
    /// Imported export payload.
    pub export: ServiceExport,
    /// Importing node.
    pub into_node: NodeID,
}

/// Export registry — services are unexported by default.
#[derive(Clone, Debug, Default)]
pub struct ExportTable {
    exported: HashMap<String, ServiceExport>,
    /// Services explicitly marked exportable.
    allow_export: HashSet<String>,
}

impl ExportTable {
    /// Create.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark a service as allowed to export.
    pub fn allow(&mut self, service: &ServiceID) {
        self.allow_export.insert(service.to_string());
    }

    /// Export an instance (fails if not allow-listed).
    pub fn export(&mut self, instance: ServiceInstance) -> Result<ServiceExport> {
        if !self.allow_export.contains(instance.service.as_str()) {
            return Err(FabricError::PolicyDenied(format!(
                "service {} is not marked for export (default unexported)",
                instance.service
            )));
        }
        let rec = ServiceExport {
            service: instance.service.clone(),
            from_node: instance.node.clone(),
            from_region: instance.region.clone(),
            network: instance.network.clone(),
            instance: instance.clone(),
            exported: true,
        };
        self.exported
            .insert(instance.service.to_string(), rec.clone());
        Ok(rec)
    }

    /// Import a remote export into local view (returns instance to register).
    pub fn import(&self, exp: ServiceExport, into_node: NodeID) -> Result<ServiceImport> {
        if !exp.exported {
            return Err(FabricError::PolicyDenied(
                "refusing to import unexported service".into(),
            ));
        }
        Ok(ServiceImport {
            export: exp,
            into_node,
        })
    }

    /// List exports.
    pub fn list(&self) -> Vec<&ServiceExport> {
        self.exported.values().collect()
    }
}

/// Serialize export to JSON (CLI `fabric export`).
pub fn export_to_json(exp: &ServiceExport) -> Result<String> {
    Ok(serde_json::to_string_pretty(exp)?)
}

/// Parse import JSON (CLI `fabric import`).
pub fn import_from_json(s: &str) -> Result<ServiceExport> {
    Ok(serde_json::from_str(s)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ServiceInstance;

    #[test]
    fn default_unexported() {
        let mut t = ExportTable::new();
        let inst = ServiceInstance::new("payments", "n1", "frankfurt", "prod", 8080);
        assert!(t.export(inst).is_err());
    }

    #[test]
    fn allow_then_export_import() {
        let mut t = ExportTable::new();
        t.allow(&"payments".into());
        let inst = ServiceInstance::new("payments", "n1", "frankfurt", "prod", 8080);
        let exp = t.export(inst).unwrap();
        let imp = t.import(exp, "n2".into()).unwrap();
        assert!(imp.export.exported);
    }
}
