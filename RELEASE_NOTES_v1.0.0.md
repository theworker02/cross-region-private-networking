# Release Notes — v1.0.0

**Date:** 2026-09-21  
**Status:** Finished acquisition release  
**License:** Proprietary — use only after written commercial license or completed acquisition ([`LICENSE`](./LICENSE))

## Highlights

- **Authoritative version 1.0.0** across Cargo workspace, docs, site, acquisition package, and evaluation scripts
- **rustls mTLS ingress** with in-process Fabric CA, WebPki client verification, and TrustStore admission/revocation
- **Complete diligence tree** under `acquisition/` for technical and commercial review
- **Honest disclosures** preserved (live multi-region not executed; no third-party pen-test; legal drafts unmarked as executed)

## License stance (one line)

No right to use, copy, modify, distribute, sublicense, or deploy in production until a written commercial license **or** completed acquisition/asset transfer.

## What buyers should run

```powershell
cargo test --workspace
.\evaluate.ps1
```

Artifacts land in `evaluation/`. Demo narrative: `docs/ACQUISITION_DEMO.md`.

## Independence

Independent project; **not affiliated with Render**. Extends the same-region private networking model conceptually; does not claim partnership.

## Supersedes

v0.5.0 remains documented in CHANGELOG as the prior acquisition-candidate milestone. **Current release is 1.0.0.**

## Disclosures

See `ACQUISITION_READINESS_REPORT.md` category **READY_WITH_DISCLOSED_ITEMS**.
