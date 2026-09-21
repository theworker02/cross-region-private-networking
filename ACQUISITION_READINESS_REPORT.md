# Acquisition readiness report

**Product:** Cross-Region Private Networking  
**Diligence package version:** 1.1.0  
**Implementation baseline:** v1.0.0 (frozen)  
**Date:** 2026-09-21

## Category: READY_WITH_DISCLOSED_ITEMS

### Evidence for readiness

| Evidence | Path / command |
|----------|----------------|
| Frozen implementation tag | `v1.0.0` |
| Expanded diligence room | `acquisition/` (v1.1.0 materials) |
| Automated tests | `cargo test --workspace` (78 passed on baseline) |
| rustls mTLS | `mtls::tests::*` VERIFIED |
| Evaluate bundle | `.\evaluate.ps1` / `./evaluate.sh` |
| Gap matrix | `docs/RENDER_GAP_MATRIX.md` + `acquisition/RENDER_GAP_MATRIX.md` |
| Integration / insertion | `acquisition/RENDER_INTEGRATION_PLAN.md`, `RENDER_INSERTION_POINTS.md` |
| Security review | `acquisition/SECURITY_REVIEW.md` |
| Internal assessment | `acquisition/SECURITY_PENTEST_INTERNAL.md` |
| Threat summary | `acquisition/THREAT_MODEL_SUMMARY.md` |
| SBOM | `acquisition/SBOM.json` |
| Transfer inventory | `acquisition/TRANSFER_INVENTORY.json` |
| Evaluation guide | `acquisition/EVALUATION_GUIDE.md` |
| License | Sale/acquisition-only `LICENSE` |
| Pages | https://theworker02.github.io/cross-region-private-networking/ |

### Disclosed items (not blockers for diligence conversation)

1. Live multi-region infrastructure exercise: **NOT EXECUTED — INFRASTRUCTURE UNAVAILABLE**  
2. No formal **external** pen-test (internal assessment provided)  
3. No patents/trademarks/executed transfer agreements in-repo (drafts only — marked NOT EXECUTED)  
4. Example IP allowlist CIDRs are not official Render ranges  

### Version narrative

- **v1.0.0** — frozen acquisition-candidate implementation + initial package  
- **v1.1.0** — substantial expansion of diligence data room, docs cross-links, and site substance — does **not** rewrite v1.0.0 history  

### Explicitly out of scope

Volume migration, DB migration, filesystem replication, deployment state transfer (Agent 2).

### Messaging compliance

README and gap matrix state Render’s same-region private networking positively and position fabric as a global extension. **Independent project; not affiliated with Render.**

### Recommendation

Suitable for **technical acquisition diligence** with the disclosed items above. Expanded `acquisition/` is intended as a serious data-room index; still not a claim of completed production multi-tenant rollout on Render.
