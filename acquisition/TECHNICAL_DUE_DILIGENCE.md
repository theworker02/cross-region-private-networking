# Technical due diligence — v1.1.0 (expanded)

## Scope

Networking / service fabric only. **No** Agent 2 storage, volume migration, or DB replication.

**Implementation baseline:** v1.0.0 freeze. **Package version:** 1.1.0 (this expanded DD narrative).

## Repository map

| Path | Role |
|------|------|
| `crates/fabric-core` | Library: identity, handshake, DNS, ingress, mtls, routing, policy, quarantine, drills, … |
| `crates/fabric-cli` | Binary `fabric`: resolve, doctor, benchmark, evaluate, quarantine, … |
| `docs/` | Engineering documentation |
| `acquisition/` | Diligence data room |
| `site/` | GitHub Pages presentation |
| `schemas/` | Event JSON schema |
| `config/` | Example CIDR configs (not official Render ranges) |
| `evaluate.ps1` / `evaluate.sh` | Acquisition smoke harness |
| `evaluation/` | Generated artifacts from evaluate |

## Architecture (summary)

```
Apps (region A) → fabric-node-A ══ rustls mTLS / fabric path ══ fabric-node-B ← Apps (region B)
                      │                                              │
                 LOCAL_NATIVE when same-region                  TrustStore + policy
```

Details: [`../docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md).

## Module inventory (fabric-core)

| Area | Modules (indicative) | Diligence notes |
|------|----------------------|-----------------|
| Identity | `identity`, `ids`, `admission` | Ed25519; TrustStore revoke |
| Session | `handshake` | Replay rejection tested |
| Naming | `dns`, `global_dns`, `namespace` | `*.internal` / `*.global.internal` |
| Ingress | `ingress`, `mtls` | CIDR + rustls mTLS |
| Routing | `routing`, `graph`, `relay`, `topology` | Path classes |
| Security | `route_security`, `quarantine`, `policy*` | Default-deny; adversarial ads |
| Ops | `evacuate`, `health`, `drills`, `sim` | Network-only evacuate |
| Platform | `connector`, `platform`, `bypass` | LOCAL_NATIVE preference |
| Control | `control_plane`, `export`, `registry` | Discovery / CP helpers |

Exact file list: browse `crates/fabric-core/src/`.

## CLI surface

See [`../docs/CLI.md`](../docs/CLI.md). Common: `resolve`, `doctor`, `benchmark`, `evaluate`, `quarantine`.

## Tests & quality gates

```text
cargo test --workspace   # 78+ unit/integration tests expected on 1.0.0 baseline
evaluate.ps1 | evaluate.sh
```

Notable security-relevant tests: handshake unknown/replay; TrustStore revoke; `mtls_accepts_valid_client` / missing cert / revoked; route_security; policy differential.

## Operations (post-close)

See `POST_TRANSFER_OPERATIONS.md`, `BUILD_REPRODUCIBILITY.md`, `SECRETS_HANDOFF_CHECKLIST.md`.

## Known technical disclosures

1. Live multi-region not executed  
2. Example CIDRs not official  
3. Demo Fabric CA is ephemeral — buyer must operate real PKI  
4. No external pen-test  

## Out of scope

Storage migration, DB cutover, filesystem sync, multi-tenant billing.
