# Technical due diligence — v1.0.0

## Scope

Networking / service fabric only. **No** Agent 2 storage, volume migration, or DB replication features.

## Code map

| Area | Crate / module | Notes |
|------|----------------|-------|
| Identity | `fabric_core::identity` | Ed25519, TrustStore |
| Handshake | `handshake` | Authn session establishment |
| DNS / names | `dns`, `global_dns` | `*.internal`, `*.global.internal` |
| Ingress | `ingress`, `mtls` | CIDR + rustls mTLS |
| Routing / policy | `routing`, policy modules | Default-deny, compiler tests |
| Quarantine | `quarantine` | Forensics preserved |
| CLI | `fabric-cli` | resolve, doctor, benchmark, evaluate, quarantine |

## Verification commands

```text
cargo test --workspace
evaluate.ps1 | evaluate.sh
```

## Maturity

v1.0.0 is an acquisition-ready **implementation package** with SIMULATED multi-region drills. It is **not** a claim of production multi-tenant SaaS operated by the seller on Render.

## Disclosures

See root `ACQUISITION_READINESS_REPORT.md`.
