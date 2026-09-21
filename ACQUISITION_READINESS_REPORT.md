# Acquisition readiness report

**Product:** Cross-Region Private Networking  
**Version:** 1.0.0  
**Date:** 2026-09-21

## Category: READY_WITH_DISCLOSED_ITEMS

### Evidence for readiness

| Evidence | Path / command |
|----------|----------------|
| Automated tests | `cargo test --workspace` |
| rustls mTLS | `mtls::tests::*` VERIFIED |
| Evaluate bundle | `.\evaluate.ps1` / `./evaluate.sh` → `evaluation/` |
| Gap matrix cited | `docs/RENDER_GAP_MATRIX.md` |
| Security review | `acquisition/SECURITY_REVIEW.md` |
| Internal assessment | `acquisition/SECURITY_PENTEST_INTERNAL.md` |
| SBOM (no UNKNOWN) | `acquisition/SBOM.json` |
| Transfer inventory | `acquisition/TRANSFER_INVENTORY.json` |
| Demo script | `docs/ACQUISITION_DEMO.md` |
| Release notes | `RELEASE_NOTES_v1.0.0.md` |
| License | Sale/acquisition-only `LICENSE` |

### Disclosed items (not blockers for diligence conversation)

1. Live multi-region infrastructure exercise: **NOT EXECUTED — INFRASTRUCTURE UNAVAILABLE**  
2. No formal **external** pen-test (internal assessment provided)  
3. No patents/trademarks/executed transfer agreements in-repo (drafts only — marked NOT EXECUTED)  
4. Example IP allowlist CIDRs are not official Render ranges  

### Closed since 0.5.0 candidate

- Production-oriented rustls mTLS terminator path (was OPEN) → **VERIFIED** in 1.0.0

### Explicitly out of scope

Volume migration, DB migration, filesystem replication, deployment state transfer (Agent 2).

### Messaging compliance

README and gap matrix state Render’s same-region private networking positively and position fabric as a global extension. **Independent project; not affiliated with Render.**

### Recommendation

Suitable for **technical acquisition diligence** at **v1.0.0** with the disclosed items above. Not a claim of completed production multi-tenant rollout on Render.
