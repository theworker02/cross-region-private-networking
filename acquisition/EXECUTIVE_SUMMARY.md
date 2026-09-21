# Executive summary — v1.0.0

**Product:** Cross-Region Private Networking  
**Release:** 1.0.0  
**Readiness:** READY_WITH_DISCLOSED_ITEMS (see root report)

## Thesis

Render’s private networking is excellent **within a region**. Multi-region customers still need identity-aware, named, policy-controlled connectivity across regions without making every service public. This fabric prefers `LOCAL_NATIVE` in-region and uses authenticated fabric paths (`FABRIC_*`) only when crossing regions.

## What ships in 1.0.0

- Rust workspace: `fabric-core` + `fabric-cli`
- Ed25519 identity, TrustStore, discovery/DNS (`*.internal` / `*.global.internal`)
- Path classification, policy, quarantine, route advertisement security
- **rustls mTLS** ingress with Fabric CA + TrustStore enforcement
- Evaluation scripts, SIMULATED multi-region drills, acquisition diligence package
- Proprietary LICENSE: **sale / acquisition only**

## What is honestly not claimed

- Live multi-region Render deployment executed in this package
- External/third-party penetration test
- Filed patents or registered trademarks (unless separately evidenced)
- Affiliation with Render

## License one-liner

No use, copy, modify, distribute, sublicense, or production deploy until written commercial license **or** completed acquisition/asset transfer.

## Next step for buyers

NDA → run `cargo test --workspace` + `evaluate.ps1` → review `acquisition/` → LOI / APA with counsel.
