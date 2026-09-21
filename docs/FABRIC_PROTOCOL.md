# Fabric Protocol (sketch)

Version constant: `PROTOCOL_VERSION = 1` in `fabric_core::types`.

Handshake: signed offer/accept with ephemeral X25519 → HKDF session key; reject unknown/revoked/expired/replay/unsupported version/network mismatch.

Ingress: optional PUBLIC_FABRIC_INGRESS_MTLS + CIDR allowlist before session use.

See `crates/fabric-core/src/handshake.rs`, `ingress.rs`.
