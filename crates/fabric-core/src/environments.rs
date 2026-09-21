//! Environment boundaries (production / staging isolation).

use crate::error::{FabricError, Result};
use crate::ids::{NetworkID, ServiceID};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Logical environment (mirrors Render-style separation conceptually).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Environment {
    /// Production.
    Production,
    /// Staging.
    Staging,
    /// Custom label.
    Custom(String),
}

impl Environment {
    /// Parse.
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "production" | "prod" => Self::Production,
            "staging" | "stage" => Self::Staging,
            other => Self::Custom(other.into()),
        }
    }

    /// Label.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Production => "production",
            Self::Staging => "staging",
            Self::Custom(s) => s.as_str(),
        }
    }
}

/// Environment isolation policy — cross-env traffic denied by default.
#[derive(Clone, Debug, Default)]
pub struct EnvironmentGuard {
    /// Explicit cross-env allows: "staging→production" keys.
    allows: HashSet<String>,
}

impl EnvironmentGuard {
    /// Create (default deny cross-env).
    pub fn new() -> Self {
        Self::default()
    }

    fn key(from: &Environment, to: &Environment) -> String {
        format!("{}→{}", from.as_str(), to.as_str())
    }

    /// Allow a specific cross-environment flow.
    pub fn allow_cross(&mut self, from: Environment, to: Environment) {
        self.allows.insert(Self::key(&from, &to));
    }

    /// Check whether `from_env` may call `to_env` for a service.
    pub fn authorize(
        &self,
        from_env: &Environment,
        to_env: &Environment,
        _service: &ServiceID,
        _network: &NetworkID,
    ) -> Result<()> {
        if from_env == to_env {
            return Ok(());
        }
        if self.allows.contains(&Self::key(from_env, to_env)) {
            return Ok(());
        }
        Err(FabricError::PolicyDenied(format!(
            "environment isolation: {} cannot reach {} (default deny cross-env)",
            from_env.as_str(),
            to_env.as_str()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_env_ok_cross_denied() {
        let g = EnvironmentGuard::new();
        assert!(g
            .authorize(
                &Environment::Production,
                &Environment::Production,
                &"api".into(),
                &"net".into()
            )
            .is_ok());
        assert!(g
            .authorize(
                &Environment::Staging,
                &Environment::Production,
                &"api".into(),
                &"net".into()
            )
            .is_err());
    }

    #[test]
    fn explicit_cross_allow() {
        let mut g = EnvironmentGuard::new();
        g.allow_cross(Environment::Staging, Environment::Production);
        assert!(g
            .authorize(
                &Environment::Staging,
                &Environment::Production,
                &"api".into(),
                &"net".into()
            )
            .is_ok());
    }
}
