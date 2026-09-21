//! Identity-based ACL policy engine with declarative language and simulator.

use crate::error::{FabricError, Result};
use crate::ids::{RegionID, ServiceID};
use crate::types::PolicyAction;
use serde::{Deserialize, Serialize};

/// Traffic direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    /// Client → server / initiator → responder.
    Egress,
    /// Inbound to local service.
    Ingress,
    /// Either.
    Both,
}

/// Declarative policy rule (source form).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PolicyRuleDecl {
    /// Optional rule name.
    pub name: Option<String>,
    /// Source service identity (`*` = any).
    pub from_service: String,
    /// Destination service identity.
    pub to_service: String,
    /// Optional source region filter.
    pub from_region: Option<String>,
    /// Optional dest region filter.
    pub to_region: Option<String>,
    /// Port (`*` or number).
    pub port: String,
    /// Protocol (`*` / http / tcp / …).
    pub protocol: String,
    /// Direction.
    pub direction: Direction,
    /// Action.
    pub action: PolicyAction,
    /// Priority (lower wins first).
    pub priority: u32,
}

/// Compiled runtime rule.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompiledRule {
    /// Name.
    pub name: String,
    /// From service or None = any.
    pub from_service: Option<ServiceID>,
    /// To service or None = any.
    pub to_service: Option<ServiceID>,
    /// From region filter.
    pub from_region: Option<RegionID>,
    /// To region filter.
    pub to_region: Option<RegionID>,
    /// Port or None = any.
    pub port: Option<u16>,
    /// Protocol or None = any.
    pub protocol: Option<String>,
    /// Direction.
    pub direction: Direction,
    /// Action.
    pub action: PolicyAction,
    /// Priority.
    pub priority: u32,
}

/// Decision receipt explaining allow/deny.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PolicyReceipt {
    /// Final action.
    pub action: PolicyAction,
    /// Matched rule name (or default-deny).
    pub matched_rule: String,
    /// Why.
    pub reason: String,
    /// Evaluated from.
    pub from_service: String,
    /// Evaluated to.
    pub to_service: String,
    /// Port.
    pub port: u16,
    /// Protocol.
    pub protocol: String,
    /// Whether default-deny applied.
    pub default_deny: bool,
}

/// Policy engine (default-deny when enabled).
#[derive(Clone, Debug, Default)]
pub struct PolicyEngine {
    rules: Vec<CompiledRule>,
    /// If true, unmatched → DENY.
    pub default_deny: bool,
    /// Activated policy version.
    pub version: u64,
}

impl PolicyEngine {
    /// Create empty engine (default-deny on).
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            default_deny: true,
            version: 0,
        }
    }

    /// Validate and compile declarations; does not activate.
    pub fn compile(decls: &[PolicyRuleDecl]) -> Result<Vec<CompiledRule>> {
        let mut out = Vec::new();
        for (i, d) in decls.iter().enumerate() {
            let name = d
                .name
                .clone()
                .unwrap_or_else(|| format!("rule-{i}"));
            let port = match d.port.as_str() {
                "*" => None,
                p => Some(
                    p.parse::<u16>()
                        .map_err(|_| FabricError::InvalidState(format!("bad port: {p}")))?,
                ),
            };
            let protocol = match d.protocol.as_str() {
                "*" => None,
                p => Some(p.to_ascii_lowercase()),
            };
            out.push(CompiledRule {
                name,
                from_service: star_svc(&d.from_service),
                to_service: star_svc(&d.to_service),
                from_region: d.from_region.as_ref().map(|r| RegionID::new(r)),
                to_region: d.to_region.as_ref().map(|r| RegionID::new(r)),
                port,
                protocol,
                direction: d.direction,
                action: d.action,
                priority: d.priority,
            });
        }
        out.sort_by_key(|r| r.priority);
        Ok(out)
    }

    /// Validate then activate a new rule set (bumps version).
    pub fn activate(&mut self, decls: &[PolicyRuleDecl]) -> Result<u64> {
        let compiled = Self::compile(decls)?;
        self.rules = compiled;
        self.version += 1;
        Ok(self.version)
    }

    /// Evaluate a flow.
    pub fn evaluate(
        &self,
        from: &ServiceID,
        to: &ServiceID,
        from_region: Option<&RegionID>,
        to_region: Option<&RegionID>,
        port: u16,
        protocol: &str,
        direction: Direction,
    ) -> PolicyReceipt {
        let proto = protocol.to_ascii_lowercase();
        for rule in &self.rules {
            if !dir_match(rule.direction, direction) {
                continue;
            }
            if let Some(fs) = &rule.from_service {
                if fs != from {
                    continue;
                }
            }
            if let Some(ts) = &rule.to_service {
                if ts != to {
                    continue;
                }
            }
            if let Some(fr) = &rule.from_region {
                if from_region != Some(fr) {
                    continue;
                }
            }
            if let Some(tr) = &rule.to_region {
                if to_region != Some(tr) {
                    continue;
                }
            }
            if let Some(p) = rule.port {
                if p != port {
                    continue;
                }
            }
            if let Some(p) = &rule.protocol {
                if p != &proto {
                    continue;
                }
            }
            return PolicyReceipt {
                action: rule.action,
                matched_rule: rule.name.clone(),
                reason: format!("matched rule '{}' → {:?}", rule.name, rule.action),
                from_service: from.to_string(),
                to_service: to.to_string(),
                port,
                protocol: proto,
                default_deny: false,
            };
        }
        let action = if self.default_deny {
            PolicyAction::Deny
        } else {
            PolicyAction::Allow
        };
        PolicyReceipt {
            action,
            matched_rule: if self.default_deny {
                "default-deny".into()
            } else {
                "default-allow".into()
            },
            reason: if self.default_deny {
                "no rule matched; default-deny".into()
            } else {
                "no rule matched; default-allow".into()
            },
            from_service: from.to_string(),
            to_service: to.to_string(),
            port,
            protocol: proto,
            default_deny: self.default_deny,
        }
    }

    /// Policy simulator: hypothetical evaluation (does not mutate).
    pub fn simulate(
        &self,
        from: &str,
        to: &str,
        port: u16,
        protocol: &str,
        from_region: Option<&str>,
        to_region: Option<&str>,
    ) -> PolicyReceipt {
        self.evaluate(
            &ServiceID::new(from),
            &ServiceID::new(to),
            from_region.map(RegionID::new).as_ref(),
            to_region.map(RegionID::new).as_ref(),
            port,
            protocol,
            Direction::Egress,
        )
    }

    /// Enforce: Err on deny.
    pub fn authorize(
        &self,
        from: &ServiceID,
        to: &ServiceID,
        port: u16,
        protocol: &str,
    ) -> Result<PolicyReceipt> {
        let r = self.evaluate(from, to, None, None, port, protocol, Direction::Egress);
        if r.action == PolicyAction::Deny {
            Err(FabricError::PolicyDenied(r.reason.clone()))
        } else {
            Ok(r)
        }
    }
}

fn star_svc(s: &str) -> Option<ServiceID> {
    if s == "*" {
        None
    } else {
        Some(ServiceID::new(s))
    }
}

fn dir_match(rule: Direction, flow: Direction) -> bool {
    matches!(
        (rule, flow),
        (Direction::Both, _)
            | (Direction::Egress, Direction::Egress)
            | (Direction::Ingress, Direction::Ingress)
    )
}

/// Parse a minimal textual DSL line:
/// `allow from checkout to payments port 443 proto http`
/// `deny from * to admin port * proto *`
pub fn parse_policy_line(line: &str) -> Result<PolicyRuleDecl> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Err(FabricError::InvalidState("empty policy line".into()));
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    // allow|deny from <svc> to <svc> [port N|*] [proto X|*] [priority N]
    if parts.len() < 5 || parts[1] != "from" || parts[3] != "to" {
        return Err(FabricError::InvalidState(format!(
            "expected 'allow|deny from <svc> to <svc> …', got: {line}"
        )));
    }
    let action = match parts[0] {
        "allow" => PolicyAction::Allow,
        "deny" => PolicyAction::Deny,
        other => {
            return Err(FabricError::InvalidState(format!(
                "unknown action: {other}"
            )))
        }
    };
    let mut port = "*".to_string();
    let mut protocol = "*".to_string();
    let mut priority = 100u32;
    let mut i = 5;
    while i < parts.len() {
        match parts[i] {
            "port" if i + 1 < parts.len() => {
                port = parts[i + 1].to_string();
                i += 2;
            }
            "proto" | "protocol" if i + 1 < parts.len() => {
                protocol = parts[i + 1].to_string();
                i += 2;
            }
            "priority" if i + 1 < parts.len() => {
                priority = parts[i + 1]
                    .parse()
                    .map_err(|_| FabricError::InvalidState("bad priority".into()))?;
                i += 2;
            }
            other => {
                return Err(FabricError::InvalidState(format!(
                    "unexpected token: {other}"
                )))
            }
        }
    }
    Ok(PolicyRuleDecl {
        name: None,
        from_service: parts[2].to_string(),
        to_service: parts[4].to_string(),
        from_region: None,
        to_region: None,
        port,
        protocol,
        direction: Direction::Egress,
        action,
        priority,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_deny_and_allow_rule() {
        let mut eng = PolicyEngine::new();
        eng.activate(&[parse_policy_line(
            "allow from checkout to payments port 8080 proto http priority 10",
        )
        .unwrap()])
        .unwrap();
        let ok = eng
            .authorize(&"checkout".into(), &"payments".into(), 8080, "http")
            .unwrap();
        assert_eq!(ok.action, PolicyAction::Allow);
        let deny = eng.authorize(&"evil".into(), &"payments".into(), 8080, "http");
        assert!(deny.is_err());
    }

    #[test]
    fn simulator_receipt() {
        let mut eng = PolicyEngine::new();
        eng.activate(&[parse_policy_line("deny from * to admin port * proto * priority 1")
            .unwrap()])
        .unwrap();
        let r = eng.simulate("checkout", "admin", 443, "https", None, None);
        assert_eq!(r.action, PolicyAction::Deny);
        assert!(!r.matched_rule.is_empty());
    }
}
