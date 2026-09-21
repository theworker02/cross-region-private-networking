# Threat model summary (buyer-facing) — v1.1.0

Full model: [`../docs/THREAT_MODEL.md`](../docs/THREAT_MODEL.md).

## Trust boundaries

| Boundary | Trust assumption |
|----------|------------------|
| App ↔ regional fabric node | Apps trust local fabric for naming/policy |
| Fabric node ↔ fabric node | Mutual authentication (identity + mTLS) |
| Cross-region underlay | **Untrusted** Internet transit |
| Render private network (same region) | Trusted for LOCAL_NATIVE paths per platform docs |
| Operator CIDR allowlists | Shared control — must be accurate |

## Primary threats & mitigations

| Threat | Mitigation in product | Residual |
|--------|----------------------|----------|
| Spoofed peer | TrustStore + mTLS | PKI ops errors |
| Replay | Handshake anti-replay tests | Protocol evolution risk |
| Revoked node rejoin | Revoke + quarantine | Clock / distribution lag |
| Attractive false routes | Route ad authentication | Misconfiguration |
| Same-region hairpin via public | LOCAL_NATIVE preference | Bugs / mis-resolve |
| Policy bypass | Default-deny + compiler tests | Incomplete rules |
| Supply chain | SBOM; recommend cargo audit | Continuous risk |

## Explicitly not covered as “proven”

- Live multi-region red-team on Render  
- External pen-test  
- Physical / insider platform admin threats beyond docs  

## Recommendation

Acceptable for acquisition technical diligence; commission external assessment before wide production.
