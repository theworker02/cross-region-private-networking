# Security review — v1.0.0

Statuses: **VERIFIED** / **MITIGATED** / **OPEN** / **NOT TESTED**

| Item | Status | Evidence |
|------|--------|----------|
| Unknown peer rejected | VERIFIED | handshake tests |
| Replay rejected | VERIFIED | handshake tests |
| Revoked cannot rejoin | VERIFIED | identity/admission/quarantine tests |
| Ingress IP allowlist | VERIFIED | ingress tests |
| rustls mTLS valid client accepted | VERIFIED | `mtls::tests::mtls_accepts_valid_client` |
| rustls mTLS missing client cert rejected | VERIFIED | `mtls::tests::mtls_rejects_missing_client_cert` |
| rustls mTLS revoked identity rejected | VERIFIED | `mtls::tests::mtls_rejects_revoked_identity_despite_tcp` |
| Default-deny policy | VERIFIED | policy tests |
| Untrusted route ads rejected | VERIFIED | route_security tests |
| Policy differential engine==reference | VERIFIED | policy_compiler randomized |
| Hostname verification never auto-disabled | VERIFIED | endpoint cert policy tests |
| Fuzz parsers no panic | VERIFIED | fuzz tests |
| Live multi-region attack surface | NOT TESTED | INFRASTRUCTURE UNAVAILABLE |
| Formal external audit / pen-test | NOT TESTED | Internal assessment only — see SECURITY_PENTEST_INTERNAL.md |
| Supply-chain malware | MITIGATED | SBOM; run `cargo audit` in networked CI |

CIDR allowlist of platform outbound ranges is **shared** → MITIGATED only when combined with mTLS + admission.
