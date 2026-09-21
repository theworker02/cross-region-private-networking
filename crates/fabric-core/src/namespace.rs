//! Global private namespace: `*.fabric.internal` (not ephemeral instance IPs).

use crate::error::{FabricError, Result};
use crate::ids::{NodeID, RegionID, ServiceID};
use serde::{Deserialize, Serialize};

/// Canonical fabric namespace name (stable across instance churn).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FabricName {
    /// Service short name (`api`, `payments`).
    pub service: ServiceID,
    /// Optional region qualifier (`virginia`, `frankfurt`).
    pub region: Option<RegionID>,
    /// Optional stable node label (not ephemeral IP).
    pub node: Option<NodeID>,
}

impl FabricName {
    /// Parse `api.fabric.internal`, `api.virginia.fabric.internal`,
    /// `api.node-17.fabric.internal`, or legacy `*.internal`.
    pub fn parse(name: &str) -> Result<Self> {
        let lower = name.trim().to_ascii_lowercase();
        if let Some(rest) = lower.strip_suffix(".fabric.internal") {
            return Self::parse_parts(rest);
        }
        if let Some(rest) = lower.strip_suffix(".internal") {
            // Legacy Phase 2/3 form — still accepted.
            return Self::parse_parts(rest);
        }
        Err(FabricError::DnsFailed(format!(
            "not a fabric namespace name: {name}"
        )))
    }

    fn parse_parts(rest: &str) -> Result<Self> {
        let parts: Vec<&str> = rest.split('.').filter(|p| !p.is_empty()).collect();
        match parts.as_slice() {
            [svc] => Ok(Self {
                service: ServiceID::new(*svc),
                region: None,
                node: None,
            }),
            [svc, mid] => {
                if mid.starts_with("node-") {
                    Ok(Self {
                        service: ServiceID::new(*svc),
                        region: None,
                        node: Some(NodeID::new(*mid)),
                    })
                } else {
                    Ok(Self {
                        service: ServiceID::new(*svc),
                        region: Some(RegionID::new(*mid)),
                        node: None,
                    })
                }
            }
            _ => Err(FabricError::DnsFailed(format!(
                "unsupported fabric name shape: {rest}"
            ))),
        }
    }

    /// Canonical FQDN in global namespace.
    pub fn fqdn(&self) -> String {
        match (&self.region, &self.node) {
            (Some(r), _) => format!("{}.{}.fabric.internal", self.service, r),
            (None, Some(n)) => format!("{}.{}.fabric.internal", self.service, n),
            (None, None) => format!("{}.fabric.internal", self.service),
        }
    }

    /// Legacy `.internal` alias.
    pub fn legacy_fqdn(&self) -> String {
        match (&self.region, &self.node) {
            (Some(r), _) => format!("{}.{}.internal", self.service, r),
            (None, Some(n)) => format!("{}.{}.internal", self.service, n),
            (None, None) => format!("{}.internal", self.service),
        }
    }
}

/// Map friendly region aliases used in demos.
pub fn normalize_region_alias(alias: &str) -> RegionID {
    match alias.to_ascii_lowercase().as_str() {
        "virginia" | "va" | "us-east" => RegionID::new("virginia"),
        "frankfurt" | "fra" | "eu-central" => RegionID::new("frankfurt"),
        "singapore" | "sin" | "asia-southeast" => RegionID::new("singapore"),
        other => RegionID::new(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_global_and_region_names() {
        let a = FabricName::parse("api.fabric.internal").unwrap();
        assert_eq!(a.service.as_str(), "api");
        assert!(a.region.is_none());
        let b = FabricName::parse("api.virginia.fabric.internal").unwrap();
        assert_eq!(b.region.as_ref().unwrap().as_str(), "virginia");
        assert_eq!(b.fqdn(), "api.virginia.fabric.internal");
    }

    #[test]
    fn legacy_internal_still_works() {
        let a = FabricName::parse("payments.internal").unwrap();
        assert_eq!(a.legacy_fqdn(), "payments.internal");
    }
}
