# Phase 4 Baseline Audit

**Date:** 2026-09-21  
**Auditor:** Agent 3 (networking/fabric)  
**Repo version at audit start:** incomplete mid-Phase-2/3 Rust workspace (no prior PHASE2/PHASE3 reports on disk)

## Pre-modification test run

| Attempt | Result |
|---------|--------|
| First `cargo test --workspace` | **FAIL** — Cargo.toml referenced missing `benches/fabric_bench.rs` |
| After fixing bench reference + adding Phase 4 modules | **PASS** — **56** unit tests, 0 failed |

### Baseline findings (before Phase 4 completion)

| Area | Status |
|------|--------|
| Legal / acquisition docs | Present (`LICENSE`, `ACQUISITION.md`, `NOTICE.md`, …) — preserved |
| Product / architecture / threat docs | Present under `docs/` — vision-stage; threat model needed join/mTLS/CIDR update |
| Rust workspace | `fabric-core` + `fabric-cli` partially implemented |
| PHASE2_REPORT / PHASE3_REPORT | **Absent** at audit start |
| Cross-region hostname resolution | Implemented in `dns.rs` with tests (VA→Frankfurt) |
| Public fabric ingress mTLS + IP allowlist | Implemented in `ingress.rs` with tests |
| `payments.internal` proof | `FabricCore::demo_va_fra` + tests |
| RTT vs public HTTPS | SIMULATED harness only (`sim::measure_rtt_overhead_harness`) |
| Storage / volumes | Correctly **not** present (out of ownership) |
| Render HTML scrape | Not present (good) |

## Constraints carried forward

- Do not implement Agent-2 storage / volume / cutover features.
- Do not invent patents, trademarks, executed NDAs, or secret material.
- Label SIMULATED vs REAL measurements honestly.
- Public fabric ingress is **not** “Render private networking.”

## Post Phase-4 integration target

v0.4.0 with global `*.fabric.internal` namespace, gateway/export/evacuate (network-only), Pages demo site, acquisition package, and PHASE4_* reports.
