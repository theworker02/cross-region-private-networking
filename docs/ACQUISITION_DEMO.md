# Acquisition Demo Script (15 minutes)

**Independent project; not affiliated with Render.**  
Label live UI sims as **SIMULATION**.

## Prep (1 min)

```powershell
.\evaluate.ps1
# or: bash ./evaluate.sh
cargo run -p fabric-cli -- demo
```

## Steps (12)

1. **State the thesis (30s):** Render already has great same-region private nets; fabric extends them globally with identity.  
2. **Show gap matrix:** open `docs/RENDER_GAP_MATRIX.md` — cite Render docs URLs.  
3. **Before:** `cargo run -p fabric-cli --` on empty bootstrap cannot resolve remote payments (show `before_fabric` narrative).  
4. **After:** `cargo run -p fabric-cli -- resolve payments.internal` → Frankfurt fabric route.  
5. **Global name:** `resolve payments.global.internal` → `FABRIC_RELAYED` with honest underlay note.  
6. **Local bypass:** register native hostname for checkout → `LOCAL_NATIVE` (Render private hostname path).  
7. **Policy:** `fabric policy test --from checkout --to payments --port 8080` ALLOW; evil→payments DENY.  
8. **Ingress:** reject bad IP / missing mTLS (`fabric ingress …`).  
9. **Quarantine:** `fabric quarantine peer` — routes gone, forensics kept.  
10. **PrivateLink complement:** `fabric evaluate` shows mock A→fabric→B→gateway→resource drill.  
11. **Failure lab:** Pages `/failure-lab` or evaluate drills (SIMULATED).  
12. **Close:** readiness category in `ACQUISITION_READINESS_REPORT.md`; no valuation discussion.

## Do not say

- “Render has no private networking.”  
- “Fabric overlay is Render private networking.”  
- Invented patents / executed NDAs.
