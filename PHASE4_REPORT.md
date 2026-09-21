# Phase 4 Report — Cross-Region Private Networking

**Version:** 0.4.0  
**Date:** 2026-09-21  
**Owner:** Networking / fabric (Agent 3)  
**Out of scope:** Storage, volume replication, journals, deployment cutover (Agent 2)

## Summary

v0.4.0 delivers a Rust fabric core (`fabric-core`) and CLI (`fabric`) with global `*.fabric.internal` namespace, cross-region VA→Frankfurt `payments` proof, public fabric ingress gate (mTLS + IP allowlist), network-only region evacuation, export/import defaults, environment isolation, policy distribution digests, receipts, Pages demo site, and acquisition package scaffolding.

## Architecture (short)

```
Apps --private--> fabric-node (per region)
fabric-node <-- mTLS (+ optional PUBLIC_FABRIC_INGRESS allowlist) --> fabric-node
Names: *.fabric.internal → fabric routes (node/region/transport)
```

Peer transit may traverse public infrastructure; apps retain private *semantics* via fabric names. This is **not** Render-native cross-region private networking.

## Package layout

```
crates/fabric-core/   library modules (identity, dns, ingress, routing, …)
crates/fabric-cli/    `fabric` binary
config/               allowlist example (not official Render CIDRs)
site/                 GitHub Pages
examples/render-global-demo/
assets/brand/
docs/                 product + HOSTNAME_RESOLUTION + audit baseline
acquisition/          diligence stubs (no invented IP claims)
```

## Exit gates

| Gate | Status |
|------|--------|
| Global private namespace | Met (`namespace`, DNS) |
| Cross-region bridge VA↔FRA | Met (demo + tests); transit labeled honestly |
| Private endpoint provider pluggable | Met (`endpoint::PrivateEndpointProvider`) |
| DNS aliasing + cert policy | Met (no auto skip-verify) |
| Bidirectional gateway optional | Met (default disabled) |
| Export/import default unexported | Met |
| Zero-trust service identity | Met (admission + policy + receipts) |
| Environment boundaries | Met (`environments`) |
| Global failover metrics hooks | Partial — failover metrics API; multi-region alternate instances tested in evacuate |
| Region evacuate network-only + dry-run | Met |
| Latency map evidence | Honest: **no fake live map**; Pages states INFRASTRUCTURE UNAVAILABLE |
| Path pin CLI | Partial — CLI stub; affinity API in routing engine |
| Maintenance / drain | Met (`DrainTracker`) |
| Policy version distribution | Met (`policy_dist`) |
| Network receipts | Met (`receipts`) |
| Render adapter V2 metadata | Partial — `RenderAdapter` bind/PORT/ephemeral + allowlist path; richer service/instance metadata fields documented in PLATFORM notes |
| Doctor V2 evidence | Met (`fabric doctor`) |
| mTLS ingress + allowlist | Met |
| payments.internal proof | Met |
| RTT overhead honest | Met — **SIMULATED** (see benchmarks JSON) |
| Threat model who can join | Met (`docs/THREAT_MODEL.md` §10) |
| Pages + brand | Met |
| Tests | **56 passed** |

## Test results

```
cargo test --workspace
56 passed; 0 failed
```

Baseline audit: `docs/audit/PHASE4_BASELINE.md`

## Benchmarks (honest)

See `PHASE4_BENCHMARKS.json`. Label: **SIMULATED**. Overhead ≈ +8ms modeled mTLS ingress vs modeled public HTTPS baseline — **not** a live multi-region measurement.  
Real multi-region: **NOT EXECUTED — INFRASTRUCTURE UNAVAILABLE**.

## Known limitations

- No live TLS listener process; ingress logic is an admission gate + session crypto suitable for embedding behind platform TLS.
- `fabric route pin` CLI is guidance; use `RoutingEngine::set_affinity` in-process.
- PNG brand assets not rasterized (SVG sources present).
- No git tag/push performed (per instructions).

## Phase 5 recommendations

- Embed ingress gate in a real rustls/tokio acceptor  
- Wire `FABRIC_BENCH_URL` live HTTPS probe  
- Expand RenderAdapter V2 with platform metadata structs only (no scraping)  
- CI secrets scan + pages deploy verification  
