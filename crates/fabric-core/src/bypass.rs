//! Path classification: LOCAL_NATIVE vs FABRIC_DIRECT vs FABRIC_RELAYED.

use crate::ids::RegionID;
use crate::ingress::IngressTransport;
use crate::types::TransportMode;
use serde::{Deserialize, Serialize};

/// How a resolved name should be reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PathClass {
    /// Same-region: use Render private hostname (native private network).
    LocalNative,
    /// Cross-region authenticated direct fabric peer path.
    FabricDirect,
    /// Cross-region via public fabric ingress or authorized relay.
    FabricRelayed,
    /// No path.
    Unavailable,
}

impl PathClass {
    /// Stable label for APIs / CLI.
    pub fn as_str(self) -> &'static str {
        match self {
            PathClass::LocalNative => "LOCAL_NATIVE",
            PathClass::FabricDirect => "FABRIC_DIRECT",
            PathClass::FabricRelayed => "FABRIC_RELAYED",
            PathClass::Unavailable => "UNAVAILABLE",
        }
    }
}

/// Result of local-bypass decision.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BypassDecision {
    /// Classification.
    pub class: PathClass,
    /// Render private hostname when LOCAL_NATIVE.
    pub native_hostname: Option<String>,
    /// Reason.
    pub reason: String,
}

/// Prefer Render same-region private hostname; otherwise fabric.
pub fn classify_path(
    local_region: &RegionID,
    target_region: &RegionID,
    native_hostname: Option<&str>,
    fabric_transport: Option<IngressTransport>,
) -> BypassDecision {
    if local_region == target_region {
        if let Some(h) = native_hostname {
            if !h.is_empty() {
                return BypassDecision {
                    class: PathClass::LocalNative,
                    native_hostname: Some(h.to_string()),
                    reason: "same region — use Render private network hostname (native); fabric does not hairpin"
                        .into(),
                };
            }
        }
        return BypassDecision {
            class: PathClass::Unavailable,
            native_hostname: None,
            reason: "same region but no native private hostname registered".into(),
        };
    }

    match fabric_transport {
        Some(IngressTransport::Direct) => BypassDecision {
            class: PathClass::FabricDirect,
            native_hostname: None,
            reason: "cross-region — authenticated FABRIC_DIRECT peer path".into(),
        },
        Some(IngressTransport::PublicFabricIngress) | Some(IngressTransport::AuthorizedRelay) => {
            BypassDecision {
                class: PathClass::FabricRelayed,
                native_hostname: None,
                reason: "cross-region — FABRIC_RELAYED (encrypted underlay may traverse public Internet; not Render private networking)".into(),
            }
        }
        None => BypassDecision {
            class: PathClass::Unavailable,
            native_hostname: None,
            reason: "cross-region and no fabric path".into(),
        },
    }
}

/// Map path class to dataplane transport enum (LOCAL_NATIVE stays conceptually Direct locally).
pub fn path_class_to_transport(class: PathClass) -> Option<TransportMode> {
    match class {
        PathClass::LocalNative | PathClass::FabricDirect => Some(TransportMode::Direct),
        PathClass::FabricRelayed => Some(TransportMode::Relayed),
        PathClass::Unavailable => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_region_uses_native() {
        let d = classify_path(
            &"virginia".into(),
            &"virginia".into(),
            Some("payments-xxxx:10000"),
            None,
        );
        assert_eq!(d.class, PathClass::LocalNative);
        assert!(d.native_hostname.unwrap().contains("payments"));
    }

    #[test]
    fn cross_region_relayed() {
        let d = classify_path(
            &"virginia".into(),
            &"frankfurt".into(),
            Some("ignored-remote-native"),
            Some(IngressTransport::PublicFabricIngress),
        );
        assert_eq!(d.class, PathClass::FabricRelayed);
        assert!(d.native_hostname.is_none());
    }
}
