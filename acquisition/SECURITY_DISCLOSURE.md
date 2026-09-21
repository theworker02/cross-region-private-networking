# Security disclosure — v1.0.0

## Reporting

See root [`../SECURITY.md`](../SECURITY.md). Contact `[CONTACT_EMAIL]`.

## Known / disclosed items

| Item | Severity | Status |
|------|----------|--------|
| Live multi-region attack surface not exercised | Info / diligence | NOT TESTED — INFRASTRUCTURE UNAVAILABLE |
| Example CIDR allowlists not official Render ranges | Medium if misused | Documented — operators must supply real ranges |
| Shared CIDR allowlist alone insufficient | Medium | MITIGATED when combined with mTLS + TrustStore |
| No external pen-test | Info | Internal assessment only |
| Fabric CA in demos is ephemeral | Info | Buyer must operate real PKI |

## No known critical remote RCE claimed or hidden

Automated tests cover handshake rejection, revocation, missing client cert, adversarial route ads, and policy default-deny. Absence of critical findings in tests is **not** a warranty.
