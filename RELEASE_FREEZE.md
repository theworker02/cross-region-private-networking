# Release freeze — v1.0.0

**Status:** FROZEN  
**Date:** 2026-09-21  
**Tag:** `v1.0.0`  
**Product:** Cross-Region Private Networking

## What “frozen” means

**v1.0.0 is the acquisition-candidate baseline.** The annotated git tag `v1.0.0` and the matching GitHub Release define the diligence artifact set for buyers.

- Do **not** silently revise v1.0.0 semantics after freeze without a new version.
- Subsequent product work should ship as **1.0.1+** or **1.1.0+** (or a new major), not as undeclared edits to the frozen baseline.
- Editorial/Pages polish that lands on the same freeze commit before Release publication is intentional and included in this tag.

## Included in the freeze

- Rust workspace `fabric-core` / `fabric-cli` at Cargo version **1.0.0**
- Proprietary LICENSE (sale / acquisition only)
- `acquisition/` diligence tree
- Docs, evaluation harness, GitHub Pages site
- Readiness: **READY_WITH_DISCLOSED_ITEMS**

## Disclosures (unchanged)

1. Live multi-region Render exercise: **NOT EXECUTED — INFRASTRUCTURE UNAVAILABLE**  
2. Internal security assessment only — **not** a third-party pen-test  
3. Legal drafts: **DRAFT FOR PROFESSIONAL LEGAL REVIEW — NOT EXECUTED**  
4. Example CIDRs are not official Render ranges  

## Independence

Independent project; **not affiliated with Render**.

## License

No use, copy, modify, distribute, sublicense, or production deploy until written commercial license or completed acquisition/asset transfer. See [`LICENSE`](./LICENSE).

## Later versions

**v1.1.0** expands the diligence data room and documentation. It does **not** un-freeze or rewrite this v1.0.0 implementation baseline. See [`RELEASE_NOTES_v1.1.0.md`](./RELEASE_NOTES_v1.1.0.md).

## Verify

```text
git checkout v1.0.0
cargo test --workspace
```

GitHub Release: https://github.com/theworker02/cross-region-private-networking/releases/tag/v1.0.0  
Pages: https://theworker02.github.io/cross-region-private-networking/
