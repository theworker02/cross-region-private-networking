# Phase 4 Security Report

**Version:** 0.4.0  
**Related:** `docs/THREAT_MODEL.md`, `PHASE3_SECURITY_REPORT.md` (if present), `SECURITY.md`

## Controls implemented in code

| Control | Status |
|---------|--------|
| Ed25519 node identity ≠ IP | Yes |
| Admit / authorize / active / revoke | Yes |
| Handshake auth, replay, version, network checks | Yes |
| Trust revocation blocks rejoin | Yes |
| Public ingress mTLS + IP allowlist | Yes (gate) |
| Default-deny policy + receipts | Yes |
| Environment isolation default deny cross-env | Yes |
| Export default deny | Yes |
| Bidirectional gateway default off | Yes |
| Policy digest refuse INVALID | Yes |
| DNS stale invalidation | Yes |
| Cert alias policy never auto skip-verify | Yes |

## Who can join (summary)

1. Cryptographic identity generated/persisted  
2. Admission AUTHORIZED (operator)  
3. Handshake success → ACTIVE  
4. Optional public ingress: allowlisted IP **and** mTLS identity  
5. REVOKED cannot rejoin by knowing an address  

CIDR allowlists of platform outbound ranges are **shared** and insufficient alone.

## Explicit non-claims

- No formal audit / pen-test completed  
- No HSM-backed CA in-repo  
- No live multi-region attack surface measurement  
- Sample CIDRs are not official Render ranges  

## Residual risk

Platform compromise, stolen authorized keys before revocation propagates, and misconfigured `allow *` policies remain operator risks.
