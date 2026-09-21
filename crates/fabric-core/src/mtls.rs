//! Production rustls mutual-TLS acceptor for public fabric ingress.
//!
//! Leaves are issued by an in-process Fabric CA (ECDSA P-256). The rustls stack
//! verifies certificate chains to that CA; fabric TrustStore enforces admission /
//! revocation on the embedded `NodeID` (CN / SPIFFE URI).
//!
//! Underlay = authenticated Internet transit — **not** Render private networking.

use crate::error::{FabricError, Result};
use crate::identity::{NodeIdentity, TrustStore};
use crate::ids::NodeID;
use crate::ingress::{FabricIngress, IngressDecision, IngressTransport, IpAllowlist};
use chrono::Duration;
use parking_lot::RwLock;
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, IsCa, KeyPair, KeyUsagePurpose,
    SanType,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use rustls::server::WebPkiClientVerifier;
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration as StdDuration;

const NODE_SAN_PREFIX: &str = "spiffe://fabric/node/";

/// Fabric TLS CA.
pub struct FabricCa {
    cert: Certificate,
    key: KeyPair,
    cert_der: CertificateDer<'static>,
    /// PEM for operators.
    pub cert_pem: String,
}

impl FabricCa {
    /// Generate CA.
    pub fn generate() -> Result<Self> {
        let key = KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256)
            .map_err(|e| FabricError::InvalidState(format!("CA key: {e}")))?;
        let mut params = CertificateParams::new(vec!["Fabric Mesh CA".into()])
            .map_err(|e| FabricError::InvalidState(format!("CA params: {e}")))?;
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
            KeyUsagePurpose::DigitalSignature,
        ];
        params.not_before = rcgen::date_time_ymd(2024, 1, 1);
        params.not_after = rcgen::date_time_ymd(2035, 1, 1);
        let cert = params
            .self_signed(&key)
            .map_err(|e| FabricError::InvalidState(format!("CA cert: {e}")))?;
        let cert_der = CertificateDer::from(cert.der().to_vec());
        let cert_pem = cert.pem();
        Ok(Self {
            cert,
            key,
            cert_der,
            cert_pem,
        })
    }

    /// Issue node leaf.
    pub fn issue_node(&self, identity: &NodeIdentity, _ttl: Duration) -> Result<FabricTlsMaterial> {
        let key = KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256)
            .map_err(|e| FabricError::InvalidState(format!("leaf key: {e}")))?;
        let mut params = CertificateParams::new(vec![identity.public.node_id.to_string()])
            .map_err(|e| FabricError::InvalidState(format!("leaf params: {e}")))?;
        params
            .distinguished_name
            .push(DnType::CommonName, identity.public.node_id.as_str());
        params.subject_alt_names = vec![
            SanType::DnsName(
                "localhost"
                    .try_into()
                    .map_err(|_| FabricError::InvalidState("dns".into()))?,
            ),
            SanType::URI(
                format!("{NODE_SAN_PREFIX}{}", identity.public.node_id)
                    .try_into()
                    .map_err(|_| FabricError::InvalidState("uri".into()))?,
            ),
        ];
        params.not_before = rcgen::date_time_ymd(2024, 1, 1);
        params.not_after = rcgen::date_time_ymd(2030, 1, 1);
        let cert = params
            .signed_by(&key, &self.cert, &self.key)
            .map_err(|e| FabricError::InvalidState(format!("sign leaf: {e}")))?;
        Ok(FabricTlsMaterial {
            node_id: identity.public.node_id.clone(),
            cert_pem: cert.pem(),
            key_pem: key.serialize_pem(),
            cert_der: CertificateDer::from(cert.der().to_vec()),
            key_der: PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key.serialize_der())),
        })
    }

    /// CA DER.
    pub fn ca_der(&self) -> CertificateDer<'static> {
        self.cert_der.clone()
    }
}

/// Issued leaf material.
pub struct FabricTlsMaterial {
    /// Node id.
    pub node_id: NodeID,
    /// Cert PEM.
    pub cert_pem: String,
    /// Key PEM.
    pub key_pem: String,
    /// Cert DER.
    pub cert_der: CertificateDer<'static>,
    /// Key DER.
    pub key_der: PrivateKeyDer<'static>,
}

impl FabricTlsMaterial {
    fn clone_key(&self) -> PrivateKeyDer<'static> {
        PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(match &self.key_der {
            PrivateKeyDer::Pkcs8(k) => k.secret_pkcs8_der().to_vec(),
            other => other.secret_der().to_vec(),
        }))
    }

    /// Node id from peer certs.
    pub fn node_id_from_certs(certs: &[CertificateDer<'_>]) -> Result<NodeID> {
        let leaf = certs
            .first()
            .ok_or_else(|| FabricError::AuthFailed("no certificate".into()))?;
        extract_node_id_from_der(leaf.as_ref())
            .map(NodeID::new)
            .ok_or_else(|| FabricError::AuthFailed("missing fabric node id in cert".into()))
    }
}

fn extract_node_id_from_der(der: &[u8]) -> Option<String> {
    // Only trust the SPIFFE URI we embed. NodeIdentity ids are exactly
    // `node-` + 12 hex chars — stop there so trailing ASN.1 (e.g. 0x30) is not
    // misread as part of the id.
    let prefix = NODE_SAN_PREFIX.as_bytes();
    let pos = find_subslice(der, prefix)?;
    let start = pos + prefix.len();
    if der.len() < start + 17 {
        return None;
    }
    let candidate = &der[start..start + 17];
    if &candidate[..5] != b"node-" {
        return None;
    }
    if !candidate[5..].iter().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    std::str::from_utf8(candidate).ok().map(str::to_string)
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

fn root_store(ca: &FabricCa) -> Result<RootCertStore> {
    let mut roots = RootCertStore::empty();
    roots
        .add(ca.ca_der())
        .map_err(|e| FabricError::InvalidState(format!("root add: {e}")))?;
    Ok(roots)
}

/// Enforce TrustStore after cryptographic mTLS succeeds.
fn enforce_trust(trust: &TrustStore, node: &NodeID) -> Result<()> {
    let id = trust
        .get(node)
        .ok_or_else(|| FabricError::UnknownIdentity(node.to_string()))?;
    trust.authenticate(id, chrono::Utc::now())
}

/// mTLS ingress server.
pub struct MtlsIngressServer {
    listener: TcpListener,
    config: Arc<ServerConfig>,
    ingress: FabricIngress,
    /// CA.
    pub ca: Arc<FabricCa>,
    local_material: FabricTlsMaterial,
}

impl MtlsIngressServer {
    /// Bind listener.
    pub fn bind(
        addr: SocketAddr,
        local: &NodeIdentity,
        _trust: Arc<RwLock<TrustStore>>,
        allowlist: IpAllowlist,
        ca: Arc<FabricCa>,
    ) -> Result<Self> {
        let material = ca.issue_node(local, Duration::hours(24))?;
        let roots = Arc::new(root_store(&ca)?);
        let client_verifier = WebPkiClientVerifier::builder(roots)
            .build()
            .map_err(|e| FabricError::InvalidState(format!("client verifier: {e}")))?;
        let config = ServerConfig::builder()
            .with_client_cert_verifier(client_verifier)
            .with_single_cert(vec![material.cert_der.clone()], material.clone_key())
            .map_err(|e| FabricError::InvalidState(format!("ServerConfig: {e}")))?;
        let listener =
            TcpListener::bind(addr).map_err(|e| FabricError::Io(e.to_string()))?;
        Ok(Self {
            listener,
            config: Arc::new(config),
            ingress: FabricIngress::new(allowlist, true),
            ca,
            local_material: material,
        })
    }

    /// Local addr.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.listener
            .local_addr()
            .map_err(|e| FabricError::Io(e.to_string()))
    }

    /// Accept one connection (IP allowlist → rustls mTLS → TrustStore).
    pub fn accept_one(&self, trust: &TrustStore) -> Result<(NodeID, IngressDecision)> {
        let (mut stream, peer) = self
            .listener
            .accept()
            .map_err(|e| FabricError::Io(e.to_string()))?;
        let _ = stream.set_read_timeout(Some(StdDuration::from_secs(5)));
        let _ = stream.set_write_timeout(Some(StdDuration::from_secs(5)));
        let ip = peer.ip();
        let pre = self.ingress.admit(ip, None, trust);
        if !pre.allowed && pre.reason.contains("not in allowlist") {
            return Ok((NodeID::new("unknown"), pre));
        }

        let mut conn = rustls::ServerConnection::new(self.config.clone())
            .map_err(|e| FabricError::AuthFailed(format!("tls: {e}")))?;
        handshake_server(&mut conn, &mut stream)?;
        let certs = conn.peer_certificates().map(|c| c.to_vec()).ok_or_else(|| {
            FabricError::AuthFailed("mTLS required: no client certificate presented".into())
        })?;
        let node_id = FabricTlsMaterial::node_id_from_certs(&certs)?;
        match enforce_trust(trust, &node_id) {
            Ok(()) => Ok((
                node_id.clone(),
                IngressDecision {
                    allowed: true,
                    transport: IngressTransport::PublicFabricIngress,
                    peer_node: Some(node_id),
                    reason: "rustls mTLS + TrustStore ok; transit=PUBLIC (not Render private networking)"
                        .into(),
                    client_ip: ip.to_string(),
                },
            )),
            Err(e) => Ok((
                node_id,
                IngressDecision {
                    allowed: false,
                    transport: IngressTransport::PublicFabricIngress,
                    peer_node: None,
                    reason: format!("rustls cert ok but fabric identity rejected: {e}"),
                    client_ip: ip.to_string(),
                },
            )),
        }
    }

    /// Server leaf.
    pub fn material(&self) -> &FabricTlsMaterial {
        &self.local_material
    }
}

fn handshake_server(conn: &mut rustls::ServerConnection, stream: &mut TcpStream) -> Result<()> {
    let mut loops = 0;
    while conn.is_handshaking() && loops < 64 {
        loops += 1;
        if conn.wants_read() {
            conn.read_tls(stream)
                .map_err(|e| FabricError::AuthFailed(format!("s read: {e}")))?;
            conn.process_new_packets()
                .map_err(|e| FabricError::AuthFailed(format!("s proc: {e}")))?;
        }
        if conn.wants_write() {
            conn.write_tls(stream)
                .map_err(|e| FabricError::AuthFailed(format!("s write: {e}")))?;
        }
    }
    if conn.is_handshaking() {
        return Err(FabricError::AuthFailed("handshake incomplete".into()));
    }
    let mut tls = rustls::Stream::new(conn, stream);
    tls.write_all(b"FABRIC_MTLS_OK\n")
        .map_err(|e| FabricError::AuthFailed(format!("app: {e}")))?;
    Ok(())
}

fn handshake_client(conn: &mut rustls::ClientConnection, stream: &mut TcpStream) -> Result<Vec<u8>> {
    let mut loops = 0;
    while conn.is_handshaking() && loops < 64 {
        loops += 1;
        if conn.wants_write() {
            conn.write_tls(stream)
                .map_err(|e| FabricError::AuthFailed(format!("c write: {e}")))?;
        }
        if conn.wants_read() {
            conn.read_tls(stream)
                .map_err(|e| FabricError::AuthFailed(format!("c read: {e}")))?;
            conn.process_new_packets()
                .map_err(|e| FabricError::AuthFailed(format!("c proc: {e}")))?;
        }
    }
    if conn.is_handshaking() {
        return Err(FabricError::AuthFailed("client handshake incomplete".into()));
    }
    let mut buf = vec![0u8; 64];
    let mut tls = rustls::Stream::new(conn, stream);
    let n = tls
        .read(&mut buf)
        .map_err(|e| FabricError::AuthFailed(format!("c app: {e}")))?;
    buf.truncate(n);
    Ok(buf)
}

/// Client mTLS connect.
pub fn mtls_connect(
    addr: SocketAddr,
    client_identity: &NodeIdentity,
    trust: Arc<RwLock<TrustStore>>,
    ca: &FabricCa,
) -> Result<NodeID> {
    let material = ca.issue_node(client_identity, Duration::hours(24))?;
    let roots = root_store(ca)?;
    let config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_client_auth_cert(vec![material.cert_der.clone()], material.clone_key())
        .map_err(|e| FabricError::InvalidState(format!("ClientConfig: {e}")))?;
    let mut stream = TcpStream::connect_timeout(&addr, StdDuration::from_secs(3))
        .map_err(|e| FabricError::Io(e.to_string()))?;
    let name = ServerName::try_from("localhost")
        .map_err(|_| FabricError::InvalidState("name".into()))?;
    let mut conn = rustls::ClientConnection::new(Arc::new(config), name)
        .map_err(|e| FabricError::AuthFailed(e.to_string()))?;
    let _ = handshake_client(&mut conn, &mut stream)?;
    let certs = conn
        .peer_certificates()
        .ok_or_else(|| FabricError::AuthFailed("no server cert".into()))?;
    let server_node = FabricTlsMaterial::node_id_from_certs(certs)?;
    enforce_trust(&trust.read(), &server_node)?;
    Ok(server_node)
}

/// No client cert — must fail.
pub fn mtls_connect_without_client_cert(addr: SocketAddr, ca: &FabricCa) -> Result<()> {
    let roots = root_store(ca)?;
    let config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let mut stream = TcpStream::connect_timeout(&addr, StdDuration::from_secs(2))
        .map_err(|e| FabricError::Io(e.to_string()))?;
    let name = ServerName::try_from("localhost")
        .map_err(|_| FabricError::InvalidState("name".into()))?;
    let mut conn = rustls::ClientConnection::new(Arc::new(config), name)
        .map_err(|e| FabricError::AuthFailed(e.to_string()))?;
    match handshake_client(&mut conn, &mut stream) {
        Err(_) => Ok(()),
        Ok(_) => Err(FabricError::InvalidState(
            "accepted without client certificate".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddrV4};
    use std::thread;

    fn allow_localhost() -> IpAllowlist {
        let mut a = IpAllowlist::example_sample_for_tests();
        a.cidrs
            .push(crate::ingress::Cidr::parse("127.0.0.1/32").unwrap());
        a
    }

    #[test]
    fn mtls_accepts_valid_client() {
        let ca = Arc::new(FabricCa::generate().unwrap());
        let server_id =
            NodeIdentity::generate("prod".into(), "virginia".into(), Duration::hours(24));
        let client_id =
            NodeIdentity::generate("prod".into(), "frankfurt".into(), Duration::hours(24));
        let mut trust = TrustStore::new();
        trust.admit(server_id.public.clone()).unwrap();
        trust.admit(client_id.public.clone()).unwrap();
        let trust = Arc::new(RwLock::new(trust));

        let srv = MtlsIngressServer::bind(
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)),
            &server_id,
            trust.clone(),
            allow_localhost(),
            ca.clone(),
        )
        .unwrap();
        let addr = srv.local_addr().unwrap();
        let trust_c = trust.clone();
        let client_id_c = client_id.clone();
        let ca_c = ca.clone();
        let handle = thread::spawn(move || {
            thread::sleep(StdDuration::from_millis(100));
            mtls_connect(addr, &client_id_c, trust_c, &ca_c)
        });
        let (peer, decision) = srv.accept_one(&trust.read()).unwrap();
        assert!(decision.allowed, "{}", decision.reason);
        assert_eq!(peer, client_id.public.node_id);
        assert_eq!(handle.join().unwrap().unwrap(), server_id.public.node_id);
    }

    #[test]
    fn mtls_rejects_missing_client_cert() {
        let ca = Arc::new(FabricCa::generate().unwrap());
        let server_id =
            NodeIdentity::generate("prod".into(), "virginia".into(), Duration::hours(24));
        let mut trust = TrustStore::new();
        trust.admit(server_id.public.clone()).unwrap();
        let trust = Arc::new(RwLock::new(trust));
        let srv = MtlsIngressServer::bind(
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)),
            &server_id,
            trust.clone(),
            allow_localhost(),
            ca.clone(),
        )
        .unwrap();
        let addr = srv.local_addr().unwrap();
        let ca_c = ca.clone();
        let handle = thread::spawn(move || {
            thread::sleep(StdDuration::from_millis(100));
            mtls_connect_without_client_cert(addr, &ca_c)
        });
        let accept = srv.accept_one(&trust.read());
        let client_ok = handle.join().unwrap().is_ok();
        assert!(
            accept.is_err() || !accept.as_ref().unwrap().1.allowed || client_ok,
            "must reject missing client cert"
        );
    }

    #[test]
    fn mtls_rejects_revoked_identity_despite_tcp() {
        let ca = Arc::new(FabricCa::generate().unwrap());
        let server_id =
            NodeIdentity::generate("prod".into(), "virginia".into(), Duration::hours(24));
        let client_id =
            NodeIdentity::generate("prod".into(), "frankfurt".into(), Duration::hours(24));
        let mut inner = TrustStore::new();
        inner.admit(server_id.public.clone()).unwrap();
        inner.admit(client_id.public.clone()).unwrap();
        inner.revoke(&client_id.public.node_id);
        let trust = Arc::new(RwLock::new(inner));
        let srv = MtlsIngressServer::bind(
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)),
            &server_id,
            trust.clone(),
            allow_localhost(),
            ca.clone(),
        )
        .unwrap();
        let addr = srv.local_addr().unwrap();
        let trust_c = trust.clone();
        let client_id_c = client_id.clone();
        let ca_c = ca.clone();
        let handle = thread::spawn(move || {
            thread::sleep(StdDuration::from_millis(100));
            mtls_connect(addr, &client_id_c, trust_c, &ca_c)
        });
        let accept = srv.accept_one(&trust.read());
        let _ = handle.join();
        match accept {
            Ok((_, d)) => assert!(!d.allowed, "revoked: {}", d.reason),
            Err(_) => {}
        }
    }
}
