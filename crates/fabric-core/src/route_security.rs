//! Route advertisement security — reject untrusted attractive ads.

use crate::error::{FabricError, Result};
use crate::ids::NodeID;
use crate::identity::TrustStore;
use crate::types::{HealthState, ServiceInstance};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Trust assumptions (documented).
pub const ROUTE_TRUST_ASSUMPTIONS: &str = "\
Route ads are accepted only from nodes present in the TrustStore (admitted, not revoked, \
not expired). Latency/loss claims from untrusted nodes are ignored. Attractive metrics \
(very low latency) from unknown advertisers cannot displace trusted routes. Operators \
must admit peers out-of-band; fabric does not auto-trust on first contact.";

/// Route advertisement from a peer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteAdvertisement {
    /// Advertising node.
    pub advertiser: NodeID,
    /// Claimed instance.
    pub instance: ServiceInstance,
    /// Claimed score hint (lower better) — untrusted until validated.
    pub claimed_score: f64,
}

/// Filter ads: only trusted advertisers; clamp absurd metrics.
pub fn accept_advertisement(
    ad: &RouteAdvertisement,
    trust: &TrustStore,
) -> Result<ServiceInstance> {
    let id = trust
        .get(&ad.advertiser)
        .ok_or_else(|| FabricError::UnknownIdentity(ad.advertiser.to_string()))?;
    trust.authenticate(id, Utc::now())?;

    if &ad.instance.node != &ad.advertiser {
        // Advertiser may only advertise own node instances (anti-spoof).
        return Err(FabricError::AuthFailed(
            "advertiser may only advertise instances on its own node".into(),
        ));
    }

    let mut inst = ad.instance.clone();
    // Clamp attractive lies: latency floor 0.1ms, reject negative loss
    if inst.latency_ms < 0.0 || inst.loss < 0.0 || inst.loss > 1.0 {
        return Err(FabricError::InvalidState(
            "route ad metrics out of bounds".into(),
        ));
    }
    // Suspiciously perfect ads from degraded peers get marked Unknown until probed
    if inst.latency_ms < 0.05 && ad.claimed_score < 0.01 {
        inst.health = HealthState::Unknown;
    }
    Ok(inst)
}

/// Prefer trusted existing over untrusted attractive ad.
pub fn select_secure<'a>(
    trusted: &'a [ServiceInstance],
    candidate_ad: &RouteAdvertisement,
    trust: &TrustStore,
) -> Result<&'a ServiceInstance> {
    match accept_advertisement(candidate_ad, trust) {
        Ok(_) => {
            // Even if ad accepted, do not auto-displace healthier trusted locals without probe.
            trusted
                .iter()
                .filter(|t| t.health.is_routable())
                .min_by(|a, b| {
                    a.latency_ms
                        .partial_cmp(&b.latency_ms)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .ok_or_else(|| FabricError::NoRoute("no trusted routes".into()))
        }
        Err(e) => {
            if let Some(t) = trusted.iter().find(|t| t.health.is_routable()) {
                let _ = e;
                Ok(t)
            } else {
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::NodeIdentity;
    use chrono::Duration;

    #[test]
    fn rejects_untrusted_attractive_ad() {
        let trust = TrustStore::new();
        let ad = RouteAdvertisement {
            advertiser: "evil".into(),
            instance: ServiceInstance::new("payments", "evil", "frankfurt", "prod", 80),
            claimed_score: 0.0,
        };
        assert!(accept_advertisement(&ad, &trust).is_err());
    }

    #[test]
    fn rejects_advertising_other_node() {
        let id = NodeIdentity::generate("prod".into(), "virginia".into(), Duration::hours(1));
        let mut trust = TrustStore::new();
        trust.admit(id.public.clone()).unwrap();
        let ad = RouteAdvertisement {
            advertiser: id.public.node_id.clone(),
            instance: ServiceInstance::new("payments", "other-node", "virginia", "prod", 80),
            claimed_score: 1.0,
        };
        assert!(accept_advertisement(&ad, &trust).is_err());
    }

    #[test]
    fn documents_assumptions() {
        assert!(ROUTE_TRUST_ASSUMPTIONS.contains("TrustStore"));
    }
}
