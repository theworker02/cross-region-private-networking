# Identity

Node identity is Ed25519-based (`NodeID` derived from key fingerprint), persisted under a data directory, rotatable with generation counters, revocable via TrustStore. Admission states: PENDING → AUTHORIZED → ACTIVE → REVOKED.

Service identity is logical (`ServiceID`); authorization is policy-based, not IP-based.

Details: `crates/fabric-core/src/identity.rs`, `admission.rs`; threat model §10.
