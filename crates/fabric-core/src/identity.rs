//! Cryptographic node identity: generation, persistence, rotation, revocation, expiry.

use crate::error::{FabricError, Result};
use crate::ids::{NetworkID, NodeID, RegionID};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Public identity material shareable with peers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicIdentity {
    /// Node identifier (derived from public key fingerprint when generated).
    pub node_id: NodeID,
    /// Ed25519 verifying key bytes.
    pub verifying_key: [u8; 32],
    /// Network membership.
    pub network: NetworkID,
    /// Home region.
    pub region: RegionID,
    /// Not-before time.
    pub not_before: DateTime<Utc>,
    /// Expiration.
    pub not_after: DateTime<Utc>,
    /// Key generation / rotation counter.
    pub generation: u64,
}

impl PublicIdentity {
    /// Fingerprint of the verifying key.
    pub fn fingerprint(&self) -> String {
        let mut h = Sha256::new();
        h.update(self.verifying_key);
        hex::encode(h.finalize())[..16].to_string()
    }

    /// Whether the identity is within its validity window at `now`.
    pub fn is_valid_at(&self, now: DateTime<Utc>) -> bool {
        now >= self.not_before && now < self.not_after
    }

    /// Verifying key handle.
    pub fn verifying_key(&self) -> Result<VerifyingKey> {
        VerifyingKey::from_bytes(&self.verifying_key).map_err(|_| FabricError::InvalidSignature)
    }
}

/// Private + public identity held by a local node.
#[derive(Clone)]
pub struct NodeIdentity {
    /// Signing key (secret).
    signing_key: SigningKey,
    /// Public view.
    pub public: PublicIdentity,
}

impl NodeIdentity {
    /// Generate a new identity for `network` / `region` valid for `ttl`.
    pub fn generate(network: NetworkID, region: RegionID, ttl: Duration) -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let vk = signing_key.verifying_key();
        let mut h = Sha256::new();
        h.update(vk.as_bytes());
        let fp = hex::encode(h.finalize());
        let node_id = NodeID::new(format!("node-{}", &fp[..12]));
        let now = Utc::now();
        Self {
            signing_key,
            public: PublicIdentity {
                node_id,
                verifying_key: *vk.as_bytes(),
                network,
                region,
                not_before: now,
                not_after: now + ttl,
                generation: 1,
            },
        }
    }

    /// Sign arbitrary message bytes.
    pub fn sign(&self, msg: &[u8]) -> Signature {
        self.signing_key.sign(msg)
    }

    /// Rotate to a new keypair, bumping generation; old public material is returned for CRL.
    pub fn rotate(&mut self, ttl: Duration) -> PublicIdentity {
        let old = self.public.clone();
        let signing_key = SigningKey::generate(&mut OsRng);
        let vk = signing_key.verifying_key();
        let now = Utc::now();
        self.signing_key = signing_key;
        self.public.verifying_key = *vk.as_bytes();
        self.public.not_before = now;
        self.public.not_after = now + ttl;
        self.public.generation += 1;
        // Keep stable node_id across rotation so peers can revoke by node + generation.
        old
    }

    /// Persist identity to a directory (secret key file + public json).
    pub fn save(&self, dir: &Path) -> Result<()> {
        fs::create_dir_all(dir)?;
        let secret_path = dir.join("node.key");
        let public_path = dir.join("node.pub.json");
        fs::write(&secret_path, self.signing_key.to_bytes())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&secret_path, fs::Permissions::from_mode(0o600));
        }
        let json = serde_json::to_string_pretty(&self.public)?;
        fs::write(public_path, json)?;
        Ok(())
    }

    /// Load identity from a directory previously written by [`Self::save`].
    pub fn load(dir: &Path) -> Result<Self> {
        let secret = fs::read(dir.join("node.key"))?;
        if secret.len() != 32 {
            return Err(FabricError::InvalidState("node.key must be 32 bytes".into()));
        }
        let mut sk_bytes = [0u8; 32];
        sk_bytes.copy_from_slice(&secret);
        let signing_key = SigningKey::from_bytes(&sk_bytes);
        let public: PublicIdentity =
            serde_json::from_str(&fs::read_to_string(dir.join("node.pub.json"))?)?;
        Ok(Self { signing_key, public })
    }
}

/// Certificate / identity trust store with revocation.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TrustStore {
    /// Known good public identities keyed by node id.
    known: HashMap<String, PublicIdentity>,
    /// Revoked node ids (cannot rejoin by address alone).
    revoked: HashSet<String>,
    /// Revoked verifying-key fingerprints (covers rotated keys).
    revoked_fingerprints: HashSet<String>,
}

impl TrustStore {
    /// Empty trust store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Admit a peer identity into the known set (join approval).
    pub fn admit(&mut self, id: PublicIdentity) -> Result<()> {
        let key = id.node_id.as_str().to_string();
        if self.revoked.contains(&key) || self.revoked_fingerprints.contains(&id.fingerprint()) {
            return Err(FabricError::RevokedIdentity(key));
        }
        if !id.is_valid_at(Utc::now()) {
            return Err(FabricError::ExpiredIdentity(key));
        }
        self.known.insert(key, id);
        Ok(())
    }

    /// Revoke a node — it cannot rejoin even if the address is known.
    pub fn revoke(&mut self, node: &NodeID) {
        let key = node.as_str().to_string();
        if let Some(id) = self.known.remove(&key) {
            self.revoked_fingerprints.insert(id.fingerprint());
        }
        self.revoked.insert(key);
    }

    /// Revoke by public key fingerprint.
    pub fn revoke_fingerprint(&mut self, fp: &str) {
        self.revoked_fingerprints.insert(fp.to_string());
        self.known
            .retain(|_, id| id.fingerprint() != fp);
    }

    /// Look up a known identity.
    pub fn get(&self, node: &NodeID) -> Option<&PublicIdentity> {
        self.known.get(node.as_str())
    }

    /// Validate peer identity for authentication (known, not revoked, not expired).
    pub fn authenticate(&self, id: &PublicIdentity, now: DateTime<Utc>) -> Result<()> {
        let key = id.node_id.as_str();
        if self.revoked.contains(key) || self.revoked_fingerprints.contains(&id.fingerprint()) {
            return Err(FabricError::RevokedIdentity(key.into()));
        }
        if !id.is_valid_at(now) {
            return Err(FabricError::ExpiredIdentity(key.into()));
        }
        match self.known.get(key) {
            None => Err(FabricError::UnknownIdentity(key.into())),
            Some(known) => {
                if known.verifying_key != id.verifying_key {
                    return Err(FabricError::AuthFailed(
                        "verifying key does not match admitted identity".into(),
                    ));
                }
                Ok(())
            }
        }
    }

    /// Verify a signature from a known peer.
    pub fn verify_signature(&self, node: &NodeID, msg: &[u8], sig: &Signature) -> Result<()> {
        let id = self
            .get(node)
            .ok_or_else(|| FabricError::UnknownIdentity(node.to_string()))?;
        self.authenticate(id, Utc::now())?;
        let vk = id.verifying_key()?;
        vk.verify(msg, sig)
            .map_err(|_| FabricError::InvalidSignature)
    }

    /// Persist trust store.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Load trust store.
    pub fn load(path: &Path) -> Result<Self> {
        Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
    }

    /// Path helper for default trust file under data dir.
    pub fn default_path(data_dir: &Path) -> PathBuf {
        data_dir.join("trust.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn generate_persist_load_roundtrip() {
        let dir = tempdir().unwrap();
        let id = NodeIdentity::generate("net-a".into(), "us-east".into(), Duration::hours(24));
        id.save(dir.path()).unwrap();
        let loaded = NodeIdentity::load(dir.path()).unwrap();
        assert_eq!(loaded.public.node_id, id.public.node_id);
        assert_eq!(loaded.public.verifying_key, id.public.verifying_key);
    }

    #[test]
    fn revoked_cannot_rejoin() {
        let mut trust = TrustStore::new();
        let id = NodeIdentity::generate("net".into(), "eu".into(), Duration::hours(1));
        trust.admit(id.public.clone()).unwrap();
        trust.revoke(&id.public.node_id);
        assert!(matches!(
            trust.admit(id.public.clone()),
            Err(FabricError::RevokedIdentity(_))
        ));
        assert!(matches!(
            trust.authenticate(&id.public, Utc::now()),
            Err(FabricError::RevokedIdentity(_))
        ));
    }

    #[test]
    fn expired_rejected() {
        let mut trust = TrustStore::new();
        let mut id = NodeIdentity::generate("net".into(), "eu".into(), Duration::hours(1));
        id.public.not_after = Utc::now() - Duration::seconds(1);
        assert!(matches!(
            trust.admit(id.public),
            Err(FabricError::ExpiredIdentity(_))
        ));
    }
}
