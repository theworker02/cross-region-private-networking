# Security — v1.0.0

## Controls

- Ed25519 node identity + TrustStore admission/revocation  
- rustls mTLS (Fabric CA, WebPkiClientVerifier, TrustStore post-check)  
- Ingress CIDR allowlist (operator-supplied; examples are not official Render ranges)  
- Default-deny policy; route advertisement authentication  
- Quarantine with forensics  

## Documents

| Doc | Role |
|-----|------|
| [`THREAT_MODEL.md`](./THREAT_MODEL.md) | Threat model |
| [`../acquisition/SECURITY_REVIEW.md`](../acquisition/SECURITY_REVIEW.md) | Status matrix |
| [`../acquisition/SECURITY_PENTEST_INTERNAL.md`](../acquisition/SECURITY_PENTEST_INTERNAL.md) | Internal assessment |
| [`../SECURITY.md`](../SECURITY.md) | Vulnerability reporting |

## Independence

Not affiliated with Render. Underlay for cross-region fabric links is authenticated Internet transit — **not** Render private networking.
