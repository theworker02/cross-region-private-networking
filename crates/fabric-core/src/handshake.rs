//! Authenticated peer handshake with session key derivation.

use crate::error::{FabricError, Result};
use crate::identity::{PublicIdentity, TrustStore};
use crate::ids::{NetworkID, NodeID, RegionID, ServiceID};
use crate::types::{PROTOCOL_VERSION, PeerInfo, TransportMode, HealthState};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, Verifier};
use hkdf::Hkdf;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashSet;
use x25519_dalek::{PublicKey as XPublic, StaticSecret};

/// Handshake offer from an initiating peer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandshakeOffer {
    /// Protocol version claimed by initiator.
    pub protocol_version: u16,
    /// Initiator public identity.
    pub identity: PublicIdentity,
    /// Services initiator may represent.
    pub services: Vec<ServiceID>,
    /// Ephemeral X25519 public key.
    pub eph_public: [u8; 32],
    /// Fresh nonce (anti-replay).
    pub nonce: [u8; 32],
    /// Wall-clock of offer creation.
    pub created_at: DateTime<Utc>,
    /// Ed25519 signature over the canonical payload.
    pub signature: Vec<u8>,
}

/// Handshake acceptance from the responder.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandshakeAccept {
    /// Protocol version agreed.
    pub protocol_version: u16,
    /// Responder public identity.
    pub identity: PublicIdentity,
    /// Services responder may represent.
    pub services: Vec<ServiceID>,
    /// Responder ephemeral X25519 public key.
    pub eph_public: [u8; 32],
    /// Echo of initiator nonce + responder nonce.
    pub nonce: [u8; 32],
    /// Responder nonce.
    pub responder_nonce: [u8; 32],
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Signature.
    pub signature: Vec<u8>,
}

/// Established session after mutual authentication.
#[derive(Clone, Debug)]
pub struct Session {
    /// Remote peer info.
    pub peer: PeerInfo,
    /// Derived session encryption key (32 bytes).
    pub session_key: [u8; 32],
    /// Local node.
    pub local_node: NodeID,
}

fn offer_bytes(o: &HandshakeOffer) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&o.protocol_version.to_be_bytes());
    buf.extend_from_slice(o.identity.node_id.as_str().as_bytes());
    buf.extend_from_slice(o.identity.network.as_str().as_bytes());
    buf.extend_from_slice(o.identity.region.as_str().as_bytes());
    buf.extend_from_slice(&o.identity.verifying_key);
    buf.extend_from_slice(&o.eph_public);
    buf.extend_from_slice(&o.nonce);
    buf.extend_from_slice(&o.created_at.timestamp().to_be_bytes());
    for s in &o.services {
        buf.extend_from_slice(s.as_str().as_bytes());
        buf.push(0);
    }
    buf
}

fn accept_bytes(a: &HandshakeAccept) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&a.protocol_version.to_be_bytes());
    buf.extend_from_slice(a.identity.node_id.as_str().as_bytes());
    buf.extend_from_slice(a.identity.network.as_str().as_bytes());
    buf.extend_from_slice(a.identity.region.as_str().as_bytes());
    buf.extend_from_slice(&a.identity.verifying_key);
    buf.extend_from_slice(&a.eph_public);
    buf.extend_from_slice(&a.nonce);
    buf.extend_from_slice(&a.responder_nonce);
    buf.extend_from_slice(&a.created_at.timestamp().to_be_bytes());
    for s in &a.services {
        buf.extend_from_slice(s.as_str().as_bytes());
        buf.push(0);
    }
    buf
}

/// Handshake engine with replay protection.
#[derive(Debug, Default)]
pub struct HandshakeEngine {
    /// Seen initiator nonces.
    seen_nonces: HashSet<[u8; 32]>,
    /// Max clock skew allowed (seconds).
    pub max_skew_secs: i64,
}

impl HandshakeEngine {
    /// Create with default 60s skew.
    pub fn new() -> Self {
        Self {
            seen_nonces: HashSet::new(),
            max_skew_secs: 60,
        }
    }

    /// Build a signed offer.
    pub fn create_offer(
        &self,
        identity: &crate::identity::NodeIdentity,
        services: Vec<ServiceID>,
    ) -> (HandshakeOffer, StaticSecret) {
        let eph = StaticSecret::random_from_rng(rand::rngs::OsRng);
        let eph_public = XPublic::from(&eph);
        let mut nonce = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        let mut offer = HandshakeOffer {
            protocol_version: PROTOCOL_VERSION,
            identity: identity.public.clone(),
            services,
            eph_public: *eph_public.as_bytes(),
            nonce,
            created_at: Utc::now(),
            signature: Vec::new(),
        };
        let sig = identity.sign(&offer_bytes(&offer));
        offer.signature = sig.to_bytes().to_vec();
        (offer, eph)
    }

    /// Validate offer and produce accept + session materials (responder side).
    pub fn accept_offer(
        &mut self,
        offer: &HandshakeOffer,
        local: &crate::identity::NodeIdentity,
        local_services: Vec<ServiceID>,
        trust: &TrustStore,
        expected_network: &NetworkID,
    ) -> Result<(HandshakeAccept, Session, StaticSecret)> {
        self.validate_offer(offer, trust, expected_network)?;
        let eph = StaticSecret::random_from_rng(rand::rngs::OsRng);
        let eph_public = XPublic::from(&eph);
        let mut responder_nonce = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut responder_nonce);
        let mut accept = HandshakeAccept {
            protocol_version: PROTOCOL_VERSION,
            identity: local.public.clone(),
            services: local_services.clone(),
            eph_public: *eph_public.as_bytes(),
            nonce: offer.nonce,
            responder_nonce,
            created_at: Utc::now(),
            signature: Vec::new(),
        };
        let sig = local.sign(&accept_bytes(&accept));
        accept.signature = sig.to_bytes().to_vec();

        let their_eph = XPublic::from(offer.eph_public);
        let shared = eph.diffie_hellman(&their_eph);
        let session_key = derive_session_key(
            shared.as_bytes(),
            &offer.nonce,
            &responder_nonce,
            &offer.identity.node_id,
            &local.public.node_id,
        );

        let peer = PeerInfo {
            node: offer.identity.node_id.clone(),
            region: offer.identity.region.clone(),
            network: offer.identity.network.clone(),
            services: offer.services.clone(),
            protocol_version: PROTOCOL_VERSION,
            transport: TransportMode::Direct,
            health: HealthState::Healthy,
            rtt_ms: 0.0,
            loss: 0.0,
            connection_count: 0,
            bytes_sent: 0,
            bytes_recv: 0,
        };
        let session = Session {
            peer,
            session_key,
            local_node: local.public.node_id.clone(),
        };
        Ok((accept, session, eph))
    }

    /// Initiator finalizes after receiving accept.
    pub fn finalize(
        &mut self,
        offer: &HandshakeOffer,
        accept: &HandshakeAccept,
        local_eph: &StaticSecret,
        local: &crate::identity::NodeIdentity,
        trust: &TrustStore,
        expected_network: &NetworkID,
    ) -> Result<Session> {
        if accept.protocol_version != PROTOCOL_VERSION {
            return Err(FabricError::UnsupportedVersion(accept.protocol_version));
        }
        if &accept.identity.network != expected_network {
            return Err(FabricError::NetworkMismatch {
                expected: expected_network.to_string(),
                got: accept.identity.network.to_string(),
            });
        }
        trust.authenticate(&accept.identity, Utc::now())?;
        if accept.nonce != offer.nonce {
            return Err(FabricError::AuthFailed("nonce mismatch".into()));
        }
        let sig_bytes: [u8; 64] = accept
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| FabricError::InvalidSignature)?;
        let sig = Signature::from_bytes(&sig_bytes);
        let vk = accept.identity.verifying_key()?;
        vk.verify(&accept_bytes(accept), &sig)
            .map_err(|_| FabricError::InvalidSignature)?;

        let their_eph = XPublic::from(accept.eph_public);
        let shared = local_eph.diffie_hellman(&their_eph);
        let session_key = derive_session_key(
            shared.as_bytes(),
            &offer.nonce,
            &accept.responder_nonce,
            &local.public.node_id,
            &accept.identity.node_id,
        );

        Ok(Session {
            peer: PeerInfo {
                node: accept.identity.node_id.clone(),
                region: accept.identity.region.clone(),
                network: accept.identity.network.clone(),
                services: accept.services.clone(),
                protocol_version: PROTOCOL_VERSION,
                transport: TransportMode::Direct,
                health: HealthState::Healthy,
                rtt_ms: 0.0,
                loss: 0.0,
                connection_count: 0,
                bytes_sent: 0,
                bytes_recv: 0,
            },
            session_key,
            local_node: local.public.node_id.clone(),
        })
    }

    fn validate_offer(
        &mut self,
        offer: &HandshakeOffer,
        trust: &TrustStore,
        expected_network: &NetworkID,
    ) -> Result<()> {
        if offer.protocol_version != PROTOCOL_VERSION {
            return Err(FabricError::UnsupportedVersion(offer.protocol_version));
        }
        if &offer.identity.network != expected_network {
            return Err(FabricError::NetworkMismatch {
                expected: expected_network.to_string(),
                got: offer.identity.network.to_string(),
            });
        }
        trust.authenticate(&offer.identity, Utc::now())?;
        let skew = (Utc::now() - offer.created_at).num_seconds().abs();
        if skew > self.max_skew_secs {
            return Err(FabricError::AuthFailed("handshake clock skew".into()));
        }
        if !self.seen_nonces.insert(offer.nonce) {
            return Err(FabricError::ReplayedHandshake);
        }
        let sig_bytes: [u8; 64] = offer
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| FabricError::InvalidSignature)?;
        let sig = Signature::from_bytes(&sig_bytes);
        let vk = offer.identity.verifying_key()?;
        vk.verify(&offer_bytes(offer), &sig)
            .map_err(|_| FabricError::InvalidSignature)?;
        Ok(())
    }
}

fn derive_session_key(
    shared: &[u8],
    n1: &[u8; 32],
    n2: &[u8; 32],
    a: &NodeID,
    b: &NodeID,
) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(n1), shared);
    let mut out = [0u8; 32];
    let mut info = Vec::new();
    info.extend_from_slice(b"fabric-session-v1");
    info.extend_from_slice(n2);
    // Order node ids lexicographically for initiator/responder symmetry.
    let (lo, hi) = if a.as_str() <= b.as_str() {
        (a.as_str(), b.as_str())
    } else {
        (b.as_str(), a.as_str())
    };
    info.extend_from_slice(lo.as_bytes());
    info.extend_from_slice(hi.as_bytes());
    hk.expand(&info, &mut out).expect("hkdf expand");
    out
}

/// Convenience: mutually authenticate two local nodes (sim / tests).
pub fn mutual_handshake(
    a: &crate::identity::NodeIdentity,
    a_services: Vec<ServiceID>,
    b: &crate::identity::NodeIdentity,
    b_services: Vec<ServiceID>,
    trust: &TrustStore,
) -> Result<(Session, Session)> {
    let network = a.public.network.clone();
    let mut eng_a = HandshakeEngine::new();
    let mut eng_b = HandshakeEngine::new();
    let (offer, eph_a) = eng_a.create_offer(a, a_services);
    let (accept, session_b, _) = eng_b.accept_offer(&offer, b, b_services, trust, &network)?;
    let session_a = eng_a.finalize(&offer, &accept, &eph_a, a, trust, &network)?;
    Ok((session_a, session_b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::NodeIdentity;
    use chrono::Duration;

    fn pair() -> (NodeIdentity, NodeIdentity, TrustStore) {
        let a = NodeIdentity::generate("prod".into(), "us-east".into(), Duration::hours(24));
        let b = NodeIdentity::generate("prod".into(), "eu-central".into(), Duration::hours(24));
        let mut trust = TrustStore::new();
        trust.admit(a.public.clone()).unwrap();
        trust.admit(b.public.clone()).unwrap();
        (a, b, trust)
    }

    #[test]
    fn happy_path_session_keys_match() {
        let (a, b, trust) = pair();
        let (sa, sb) = mutual_handshake(&a, vec![], &b, vec!["payments".into()], &trust).unwrap();
        assert_eq!(sa.session_key, sb.session_key);
        assert_eq!(sa.peer.node, b.public.node_id);
        assert_eq!(sb.peer.node, a.public.node_id);
    }

    #[test]
    fn rejects_unknown_peer() {
        let (a, b, trust) = pair();
        let stranger =
            NodeIdentity::generate("prod".into(), "us-west".into(), Duration::hours(24));
        let mut eng = HandshakeEngine::new();
        let (offer, _) = eng.create_offer(&stranger, vec![]);
        let err = eng
            .accept_offer(&offer, &b, vec![], &trust, &a.public.network)
            .err()
            .expect("should fail");
        assert!(matches!(err, FabricError::UnknownIdentity(_)));
    }

    #[test]
    fn rejects_replay() {
        let (a, b, trust) = pair();
        let mut eng = HandshakeEngine::new();
        let (offer, _) = eng.create_offer(&a, vec![]);
        eng.accept_offer(&offer, &b, vec![], &trust, &a.public.network)
            .unwrap();
        let err = eng
            .accept_offer(&offer, &b, vec![], &trust, &a.public.network)
            .err()
            .expect("should fail");
        assert_eq!(err, FabricError::ReplayedHandshake);
    }

    #[test]
    fn rejects_revoked() {
        let (a, b, mut trust) = pair();
        trust.revoke(&a.public.node_id);
        let mut eng = HandshakeEngine::new();
        let (offer, _) = eng.create_offer(&a, vec![]);
        let err = eng
            .accept_offer(&offer, &b, vec![], &trust, &a.public.network)
            .err()
            .expect("should fail");
        assert!(matches!(err, FabricError::RevokedIdentity(_)));
    }

    #[test]
    fn rejects_bad_version() {
        let (a, b, trust) = pair();
        let mut eng = HandshakeEngine::new();
        let (mut offer, _) = eng.create_offer(&a, vec![]);
        offer.protocol_version = 99;
        // resign with wrong version still fails version check before sig... actually we check version first
        let err = eng
            .accept_offer(&offer, &b, vec![], &trust, &a.public.network)
            .err()
            .expect("should fail");
        assert!(matches!(err, FabricError::UnsupportedVersion(99)));
    }
}
