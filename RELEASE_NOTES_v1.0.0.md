# Release Notes — v1.0.0

**Date:** 2026-09-21  
**Status:** FROZEN acquisition candidate  
**License:** Proprietary — use only after written commercial license or completed acquisition ([`LICENSE`](./LICENSE))  
**Freeze:** [`RELEASE_FREEZE.md`](./RELEASE_FREEZE.md)

## Thesis

Render’s private networking is excellent **within a region**. This product extends that model with global names, rustls mTLS identity, path classification (`LOCAL_NATIVE` / `FABRIC_*`), and policy for cross-region service connectivity — without claiming affiliation with Render.

## Highlights

- Authoritative **1.0.0** across Cargo workspace, docs, site, acquisition package, and evaluation scripts
- **rustls mTLS** ingress (Fabric CA, WebPki client verification, TrustStore admission/revocation)
- Global hostname resolution with in-region **LOCAL_NATIVE** preference
- Policy, quarantine, route advertisement security, network-only evacuate drills
- Complete **`acquisition/`** diligence tree (IP, SBOM, transfer, security, legal drafts)
- GitHub Pages presentation: demo (SIMULATION), About, Target: Render, Contact, Due diligence
- Evaluate harness: `evaluate.ps1` / `evaluate.sh` → `evaluation/`

## License (one line)

No right to use, copy, modify, distribute, sublicense, or deploy in production until a written commercial license **or** completed acquisition/asset transfer.

## What buyers should run

```powershell
git checkout v1.0.0
cargo test --workspace
.\evaluate.ps1
```

## Links

| Resource | URL |
|----------|-----|
| Repository | https://github.com/theworker02/cross-region-private-networking |
| Pages | https://theworker02.github.io/cross-region-private-networking/ |
| Demo (SIMULATION) | https://theworker02.github.io/cross-region-private-networking/demo.html |
| Target: Render | https://theworker02.github.io/cross-region-private-networking/render.html |
| Contact | https://theworker02.github.io/cross-region-private-networking/contact.html |
| Readiness | [`ACQUISITION_READINESS_REPORT.md`](./ACQUISITION_READINESS_REPORT.md) |
| Acquisition package | [`acquisition/`](./acquisition/) |

## Independence

Independent project; **not affiliated with Render**.

## Disclosures

See readiness category **READY_WITH_DISCLOSED_ITEMS**:

- Live multi-region: **NOT EXECUTED — INFRASTRUCTURE UNAVAILABLE**
- No third-party pen-test (internal assessment only)
- Legal drafts **NOT EXECUTED**
- Example CIDRs are not official Render ranges

## Supersedes

v0.5.0 remains in CHANGELOG as the prior candidate milestone. **Authoritative current release is 1.0.0 (frozen).**
