//! Public fabric ingress: mutual TLS + IP allowlist.
//!
//! This is **encrypted authenticated transit over a public path** with private
//! semantics for apps. It is **not** a private network. Peers may use:
//! DIRECT → PUBLIC_FABRIC_INGRESS (mTLS+allowlist) → AUTHORIZED_RELAY.

use crate::error::{FabricError, Result};
use crate::identity::{PublicIdentity, TrustStore};
use crate::ids::NodeID;
use crate::types::TransportMode;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::IpAddr;
use std::path::Path;
use std::str::FromStr;

/// How a peer connection arrived.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IngressTransport {
    /// Authenticated direct peer reachability (private path or hole-punch).
    Direct,
    /// Hardened public fabric ingress (mTLS + allowlist) over the Internet.
    PublicFabricIngress,
    /// Authorized relay fallback.
    AuthorizedRelay,
}

impl IngressTransport {
    /// Map to dataplane transport label (never calls ingress "private").
    pub fn as_transport_mode(self) -> TransportMode {
        match self {
            IngressTransport::Direct => TransportMode::Direct,
            IngressTransport::PublicFabricIngress | IngressTransport::AuthorizedRelay => {
                TransportMode::Relayed
            }
        }
    }

    /// Operator-facing label — honest about public transit.
    pub fn label(self) -> &'static str {
        match self {
            IngressTransport::Direct => "DIRECT",
            IngressTransport::PublicFabricIngress => "PUBLIC_FABRIC_INGRESS_MTLS",
            IngressTransport::AuthorizedRelay => "AUTHORIZED_RELAY",
        }
    }
}

/// IPv4/IPv6 CIDR (prefix).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cidr {
    /// Network address.
    pub addr: IpAddr,
    /// Prefix length.
    pub prefix_len: u8,
}

impl Cidr {
    /// Parse `a.b.c.d/n` or `v6/n`.
    pub fn parse(s: &str) -> Result<Self> {
        let (addr_s, pref_s) = s
            .split_once('/')
            .ok_or_else(|| FabricError::InvalidState(format!("CIDR requires /prefix: {s}")))?;
        let addr = IpAddr::from_str(addr_s.trim())
            .map_err(|e| FabricError::InvalidState(format!("bad CIDR addr: {e}")))?;
        let prefix_len: u8 = pref_s
            .trim()
            .parse()
            .map_err(|_| FabricError::InvalidState(format!("bad prefix: {pref_s}")))?;
        match addr {
            IpAddr::V4(_) if prefix_len > 32 => {
                return Err(FabricError::InvalidState("IPv4 prefix > 32".into()));
            }
            IpAddr::V6(_) if prefix_len > 128 => {
                return Err(FabricError::InvalidState("IPv6 prefix > 128".into()));
            }
            _ => {}
        }
        Ok(Self { addr, prefix_len })
    }

    /// Whether `ip` is contained in this CIDR.
    pub fn contains(&self, ip: IpAddr) -> bool {
        match (self.addr, ip) {
            (IpAddr::V4(n), IpAddr::V4(x)) => {
                let mask = if self.prefix_len == 0 {
                    0u32
                } else {
                    u32::MAX << (32 - self.prefix_len as u32)
                };
                (u32::from(n) & mask) == (u32::from(x) & mask)
            }
            (IpAddr::V6(n), IpAddr::V6(x)) => {
                let n = u128::from(n);
                let x = u128::from(x);
                let mask = if self.prefix_len == 0 {
                    0u128
                } else {
                    u128::MAX << (128 - self.prefix_len as u32)
                };
                (n & mask) == (x & mask)
            }
            _ => false,
        }
    }
}

impl FromStr for Cidr {
    type Err = FabricError;
    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

/// IP allowlist configuration.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct IpAllowlist {
    /// Allowed CIDRs.
    pub cidrs: Vec<Cidr>,
    /// Human note: where ranges were sourced (docs URL / dashboard export).
    pub source_note: String,
    /// If true, empty list denies all; if false and empty, allow-all (dev only).
    pub deny_when_empty: bool,
}

impl IpAllowlist {
    /// Restrictive empty allowlist (deny all until configured).
    pub fn deny_all() -> Self {
        Self {
            cidrs: Vec::new(),
            source_note: "deny-all until configured".into(),
            deny_when_empty: true,
        }
    }

    /// Load from JSON file. See `config/render_outbound_cidrs.example.json`.
    pub fn load_file(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)?;
        let v: IpAllowlist = serde_json::from_str(&raw)?;
        Ok(v)
    }

    /// Parse from env `FABRIC_IP_ALLOWLIST` as comma-separated CIDRs.
    /// Optional `FABRIC_IP_ALLOWLIST_SOURCE` for provenance note.
    pub fn from_env() -> Result<Option<Self>> {
        match std::env::var("FABRIC_IP_ALLOWLIST") {
            Ok(v) if !v.trim().is_empty() => {
                let mut cidrs = Vec::new();
                for part in v.split(',') {
                    let p = part.trim();
                    if !p.is_empty() {
                        cidrs.push(Cidr::parse(p)?);
                    }
                }
                let source_note = std::env::var("FABRIC_IP_ALLOWLIST_SOURCE").unwrap_or_else(|_| {
                    "FABRIC_IP_ALLOWLIST env (operator-supplied; not vendor-official)".into()
                });
                Ok(Some(Self {
                    cidrs,
                    source_note,
                    deny_when_empty: true,
                }))
            }
            _ => Ok(None),
        }
    }

    /// Sample / placeholder ranges for **local tests only** — NOT claimed as
    /// official Render outbound ranges. Operators must replace from Render docs
    /// or dashboard before production.
    pub fn example_sample_for_tests() -> Self {
        Self {
            cidrs: vec![
                Cidr::parse("10.0.0.0/8").unwrap(),
                Cidr::parse("127.0.0.1/32").unwrap(),
                Cidr::parse("192.0.2.0/24").unwrap(), // TEST-NET-1 (RFC 5737)
            ],
            source_note: "EXAMPLE/TEST ONLY — replace with Render outbound CIDRs from https://render.com/docs or dashboard; these are RFC1918/TEST-NET samples, not official Render ranges".into(),
            deny_when_empty: true,
        }
    }

    /// Check client IP.
    pub fn permits(&self, ip: IpAddr) -> bool {
        if self.cidrs.is_empty() {
            return !self.deny_when_empty;
        }
        self.cidrs.iter().any(|c| c.contains(ip))
    }
}

/// Result of ingress admission check.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IngressDecision {
    /// Accepted?
    pub allowed: bool,
    /// Transport used if allowed.
    pub transport: IngressTransport,
    /// Peer node if mTLS identity verified.
    pub peer_node: Option<NodeID>,
    /// Denial / accept reason.
    pub reason: String,
    /// Client IP observed.
    pub client_ip: String,
}

/// Public fabric ingress gate: mTLS identity + IP allowlist.
#[derive(Clone, Debug)]
pub struct FabricIngress {
    /// Allowlist.
    pub allowlist: IpAllowlist,
    /// Whether ingress listener is enabled.
    pub enabled: bool,
}

impl FabricIngress {
    /// Create ingress gate.
    pub fn new(allowlist: IpAllowlist, enabled: bool) -> Self {
        Self { allowlist, enabled }
    }

    /// Evaluate an inbound connection attempt.
    ///
    /// `client_identity` must be present for mTLS success (presented client cert
    /// mapped to [`PublicIdentity`]). Missing identity ⇒ reject.
    pub fn admit(
        &self,
        client_ip: IpAddr,
        client_identity: Option<&PublicIdentity>,
        trust: &TrustStore,
    ) -> IngressDecision {
        if !self.enabled {
            return IngressDecision {
                allowed: false,
                transport: IngressTransport::PublicFabricIngress,
                peer_node: None,
                reason: "public fabric ingress disabled".into(),
                client_ip: client_ip.to_string(),
            };
        }

        if !self.allowlist.permits(client_ip) {
            return IngressDecision {
                allowed: false,
                transport: IngressTransport::PublicFabricIngress,
                peer_node: None,
                reason: format!(
                    "IP {client_ip} not in allowlist (source: {})",
                    self.allowlist.source_note
                ),
                client_ip: client_ip.to_string(),
            };
        }

        let Some(id) = client_identity else {
            return IngressDecision {
                allowed: false,
                transport: IngressTransport::PublicFabricIngress,
                peer_node: None,
                reason: "mTLS required: no client certificate / identity presented".into(),
                client_ip: client_ip.to_string(),
            };
        };

        match trust.authenticate(id, Utc::now()) {
            Ok(()) => IngressDecision {
                allowed: true,
                transport: IngressTransport::PublicFabricIngress,
                peer_node: Some(id.node_id.clone()),
                reason: format!(
                    "mTLS ok for {}; IP allowlist ok; transit=PUBLIC (not a private network)",
                    id.node_id
                ),
                client_ip: client_ip.to_string(),
            },
            Err(e) => IngressDecision {
                allowed: false,
                transport: IngressTransport::PublicFabricIngress,
                peer_node: None,
                reason: format!("mTLS identity rejected: {e}"),
                client_ip: client_ip.to_string(),
            },
        }
    }

    /// Prefer DIRECT; else try public ingress; else relay hint.
    pub fn select_transport(
        &self,
        direct_ok: bool,
        ingress_ok: bool,
        relay_authorized: bool,
    ) -> Result<IngressTransport> {
        if direct_ok {
            return Ok(IngressTransport::Direct);
        }
        if ingress_ok && self.enabled {
            return Ok(IngressTransport::PublicFabricIngress);
        }
        if relay_authorized {
            return Ok(IngressTransport::AuthorizedRelay);
        }
        Err(FabricError::RelayUnavailable(
            "no DIRECT, PUBLIC_FABRIC_INGRESS, or AUTHORIZED_RELAY path".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{NodeIdentity, TrustStore};
    use chrono::Duration;
    use std::net::Ipv4Addr;

    #[test]
    fn cidr_contains() {
        let c = Cidr::parse("192.0.2.0/24").unwrap();
        assert!(c.contains(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 10))));
        assert!(!c.contains(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 1))));
    }

    #[test]
    fn reject_outside_allowlist_even_with_valid_cert() {
        let id = NodeIdentity::generate("prod".into(), "us-east".into(), Duration::hours(1));
        let mut trust = TrustStore::new();
        trust.admit(id.public.clone()).unwrap();
        let ingress = FabricIngress::new(IpAllowlist::example_sample_for_tests(), true);
        // 198.51.100.1 is TEST-NET-2 — not in sample allowlist
        let d = ingress.admit(
            IpAddr::V4(Ipv4Addr::new(198, 51, 100, 1)),
            Some(&id.public),
            &trust,
        );
        assert!(!d.allowed);
        assert!(d.reason.contains("not in allowlist"));
    }

    #[test]
    fn reject_missing_mtls_even_on_allowed_ip() {
        let trust = TrustStore::new();
        let ingress = FabricIngress::new(IpAllowlist::example_sample_for_tests(), true);
        let d = ingress.admit(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 5)), None, &trust);
        assert!(!d.allowed);
        assert!(d.reason.contains("mTLS"));
    }

    #[test]
    fn accept_mtls_and_allowlisted_ip() {
        let id = NodeIdentity::generate("prod".into(), "eu-central".into(), Duration::hours(1));
        let mut trust = TrustStore::new();
        trust.admit(id.public.clone()).unwrap();
        let ingress = FabricIngress::new(IpAllowlist::example_sample_for_tests(), true);
        let d = ingress.admit(
            IpAddr::V4(Ipv4Addr::new(192, 0, 2, 50)),
            Some(&id.public),
            &trust,
        );
        assert!(d.allowed);
        assert_eq!(d.transport, IngressTransport::PublicFabricIngress);
        assert!(d.reason.contains("PUBLIC"));
    }

    #[test]
    fn revoked_rejected_on_ingress() {
        let id = NodeIdentity::generate("prod".into(), "eu-central".into(), Duration::hours(1));
        let mut trust = TrustStore::new();
        trust.admit(id.public.clone()).unwrap();
        trust.revoke(&id.public.node_id);
        let ingress = FabricIngress::new(IpAllowlist::example_sample_for_tests(), true);
        let d = ingress.admit(
            IpAddr::V4(Ipv4Addr::new(192, 0, 2, 50)),
            Some(&id.public),
            &trust,
        );
        assert!(!d.allowed);
    }
}
