//! Fabric error types.

use thiserror::Error;

/// Result alias for fabric operations.
pub type Result<T> = std::result::Result<T, FabricError>;

/// Errors produced by the fabric core.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FabricError {
    /// Identity is unknown to the network.
    #[error("unknown identity: {0}")]
    UnknownIdentity(String),

    /// Identity has been revoked.
    #[error("revoked identity: {0}")]
    RevokedIdentity(String),

    /// Identity or credential has expired.
    #[error("expired identity: {0}")]
    ExpiredIdentity(String),

    /// Cryptographic signature failed verification.
    #[error("invalid signature")]
    InvalidSignature,

    /// Handshake nonce was already seen (replay).
    #[error("replayed handshake")]
    ReplayedHandshake,

    /// Protocol version is not supported.
    #[error("unsupported protocol version: {0}")]
    UnsupportedVersion(u16),

    /// Peer is not a member of the expected network.
    #[error("network mismatch: expected {expected}, got {got}")]
    NetworkMismatch {
        /// Expected network id.
        expected: String,
        /// Peer's claimed network id.
        got: String,
    },

    /// Policy denied the request.
    #[error("policy denied: {0}")]
    PolicyDenied(String),

    /// No route available.
    #[error("no route: {0}")]
    NoRoute(String),

    /// DNS name could not be resolved to a live instance.
    #[error("dns resolution failed: {0}")]
    DnsFailed(String),

    /// Relay path is not authorized or unavailable.
    #[error("relay unavailable: {0}")]
    RelayUnavailable(String),

    /// Persistence / IO failure.
    #[error("io error: {0}")]
    Io(String),

    /// Invalid state or configuration.
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// Authentication failed generically.
    #[error("authentication failed: {0}")]
    AuthFailed(String),
}

impl From<std::io::Error> for FabricError {
    fn from(e: std::io::Error) -> Self {
        FabricError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for FabricError {
    fn from(e: serde_json::Error) -> Self {
        FabricError::Io(e.to_string())
    }
}
