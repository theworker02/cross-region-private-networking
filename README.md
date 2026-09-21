# Cross-Region Private Networking

<p align="center">
  <img src="assets/brand/logo.svg" alt="Cross-Region Private Networking" width="320"/>
</p>

<p align="center">
  <img alt="version" src="https://img.shields.io/badge/version-1.0.0-1F7A6B"/>
  <img alt="ci" src="https://img.shields.io/badge/CI-cargo%20test-1F7A6B"/>
  <img alt="tests" src="https://img.shields.io/badge/tests-workspace%20green-1F7A6B"/>
  <img alt="license" src="https://img.shields.io/badge/license-proprietary-0B1F2A"/>
  <img alt="docs" src="https://img.shields.io/badge/docs-complete-1F7A6B"/>
  <img alt="pages" src="https://img.shields.io/badge/GitHub%20Pages-site%2F-0B1F2A"/>
</p>

**Global identity-aware private fabric that extends Render’s same-region private networking across regions.**

> **Independent project; not affiliated with Render.**  
> Render already provides excellent same-region private networks. This product adds cross-region names, rustls mTLS identity, path classification, and policy — preferring native private hostnames in-region (`LOCAL_NATIVE`).

| | |
|---|---|
| **Version** | **1.0.0** acquisition release |
| **License** | Proprietary — use only after written commercial license **or** completed acquisition ([`LICENSE`](./LICENSE)) |
| **Evaluate** | `.\evaluate.ps1` or `./evaluate.sh` → `evaluation/` |
| **Readiness** | [`ACQUISITION_READINESS_REPORT.md`](./ACQUISITION_READINESS_REPORT.md) |
| **Diligence** | [`acquisition/`](./acquisition/) |

## Problem

Render private networking is **region-scoped**. Workloads in Virginia cannot privately address services in Frankfurt over Render’s internal network alone. Teams often fall back to public networking plus app-level auth — more attack surface, weaker discovery/health fabric semantics.

## Architecture (one glance)

```
[VA private net]                         [FRA private net]
  apps → fabric-node-va ══ rustls mTLS ══ fabric-node-fra ← apps
           │                                      │
     *.internal / *.global.internal          discovery + policy
     LOCAL_NATIVE in-region · FABRIC_* cross-region
```

Path classes: `LOCAL_NATIVE` · `FABRIC_DIRECT` · `FABRIC_RELAYED`  
See [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md).

## Quick start

```powershell
cargo test --workspace
.\evaluate.ps1
cargo run -p fabric-cli -- resolve payments.global.internal
cargo run -p fabric-cli -- doctor
```

Full walkthrough: [`docs/QUICKSTART.md`](./docs/QUICKSTART.md) · CLI reference: [`docs/CLI.md`](./docs/CLI.md)

## Demo

SIMULATED multi-region VA↔FRA (and optional SIN) drill with labeled receipts — see [`docs/ACQUISITION_DEMO.md`](./docs/ACQUISITION_DEMO.md) and [`docs/DEMO.md`](./docs/DEMO.md).  
Live multi-region Render infrastructure: **NOT EXECUTED — INFRASTRUCTURE UNAVAILABLE** ([`docs/LIVE_MULTI_REGION.md`](./docs/LIVE_MULTI_REGION.md)).

## Render fit

Extends the same-region private network model with global names and authenticated fabric paths. **Not affiliated with Render.** Gap matrix: [`docs/RENDER_GAP_MATRIX.md`](./docs/RENDER_GAP_MATRIX.md). Integration plan: [`docs/RENDER_INTEGRATION_ARCHITECTURE.md`](./docs/RENDER_INTEGRATION_ARCHITECTURE.md).

## Security

- Ed25519 identity + TrustStore admission / revocation  
- Production-oriented **rustls mTLS** ingress (Fabric CA + WebPkiClientVerifier + TrustStore post-check)  
- CIDR allowlist + default-deny policy + route advertisement security + quarantine  

Threat model: [`docs/THREAT_MODEL.md`](./docs/THREAT_MODEL.md) · Security notes: [`docs/SECURITY.md`](./docs/SECURITY.md) · Internal assessment: [`acquisition/SECURITY_PENTEST_INTERNAL.md`](./acquisition/SECURITY_PENTEST_INTERNAL.md)

## Benchmarks

Honest SIMULATED RTT / resolve / policy timings via `fabric benchmark` and `evaluation/benchmark.json`. See [`docs/BENCHMARKING.md`](./docs/BENCHMARKING.md).

## Documentation

| Doc | Topic |
|-----|--------|
| [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) | Topology & components |
| [`docs/QUICKSTART.md`](./docs/QUICKSTART.md) | Get running |
| [`docs/CLI.md`](./docs/CLI.md) | `fabric` commands |
| [`docs/FABRIC_PROTOCOL.md`](./docs/FABRIC_PROTOCOL.md) | Protocol |
| [`docs/IDENTITY.md`](./docs/IDENTITY.md) | Identity |
| [`docs/ROUTING.md`](./docs/ROUTING.md) | Routing |
| [`docs/POLICY.md`](./docs/POLICY.md) | Policy |
| [`docs/HOSTNAME_RESOLUTION.md`](./docs/HOSTNAME_RESOLUTION.md) | DNS / names |
| [`site/`](./site/) | Acquisition presentation (Pages) |

## Acquisition notice

This repository is a **v1.0.0 acquisition package**. Diligence materials live under [`acquisition/`](./acquisition/). One-pager: [`ACQUISITION.md`](./ACQUISITION.md).  
**No production use, redistribution, or sublicensing** until a written commercial license or completed acquisition/asset transfer.

## License

**Proprietary — sale / acquisition only.** See [`LICENSE`](./LICENSE).

- No grant to use, copy, modify, distribute, sublicense, or deploy in production by default  
- Evaluation only under written NDA if granted  
- Executed deal agreements supersede this LICENSE  
- Not legal advice; drafts in `acquisition/legal-review/` are **not** executed  

---

*© 2026 [Rightsholder]. All rights reserved. Independent project; not affiliated with Render.*
