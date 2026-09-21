//! Strongly-typed fabric identifiers.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Cryptographic / logical node identity (not an IP address).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeID(String);

impl NodeID {
    /// Create a node ID from a string label.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Generate a fresh random node ID.
    pub fn generate() -> Self {
        Self(format!("node-{}", Uuid::new_v4()))
    }

    /// Borrow the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NodeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for NodeID {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Logical service identity (e.g. `payments`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceID(String);

impl ServiceID {
    /// Create a service ID.
    pub fn new(s: impl Into<String>) -> Self {
        let raw = s.into();
        let name = raw
            .strip_suffix(".internal")
            .unwrap_or(&raw)
            .split('.')
            .next()
            .unwrap_or(&raw)
            .to_string();
        Self(name)
    }

    /// Borrow the canonical short name.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// `payments.internal` form.
    pub fn fqdn(&self) -> String {
        format!("{}.internal", self.0)
    }
}

impl fmt::Display for ServiceID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ServiceID {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Region label (e.g. `us-east`, `eu-central`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegionID(String);

impl RegionID {
    /// Create a region ID.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Borrow the string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RegionID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for RegionID {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Logical fabric / mesh network identity.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkID(String);

impl NetworkID {
    /// Create a network ID.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Borrow the string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NetworkID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for NetworkID {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_id_strips_internal_suffix() {
        assert_eq!(ServiceID::new("payments.internal").as_str(), "payments");
        assert_eq!(ServiceID::new("payments.us-east.internal").as_str(), "payments");
        assert_eq!(ServiceID::new("payments").fqdn(), "payments.internal");
    }
}
