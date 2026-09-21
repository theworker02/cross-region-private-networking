# Executive summary — v1.1.0

**Product:** Cross-Region Private Networking (Fabric)  
**Diligence package:** 1.1.0  
**Implementation baseline:** v1.0.0 (frozen)  
**Readiness:** READY_WITH_DISCLOSED_ITEMS

## Thesis (one paragraph)

Render already provides excellent **same-region** private networking. Multi-region customers still need identity-aware, named, policy-controlled connectivity across regions without making every service a public endpoint. This fabric prefers `LOCAL_NATIVE` (Render private hostnames) in-region and uses authenticated `FABRIC_*` paths only when crossing regions. It is an **independent** project — **not affiliated with Render** — positioned as a complementary capability for acquisition or commercial license by a platform that wants that extension.

## What an acquirer gets

| Layer | Contents |
|-------|----------|
| Software | Rust workspace (`fabric-core`, `fabric-cli`) — identity, discovery/DNS, rustls mTLS, policy, quarantine, drills |
| Docs & know-how | Architecture, threat model, Render gap matrix, integration/insertion plans, demo scripts |
| Diligence room | This `acquisition/` tree — IP, SBOM, security, transfer, legal **drafts** |
| Brand | Working title + SVG assets (no registered TM claimed herein) |
| Presentation | GitHub Pages site (SIMULATION demo, target narrative, contact) |

## What an acquirer does **not** get from this package alone

- Executed APA / license / patent assignment  
- Production multi-tenant SaaS operated by seller on Render  
- Proven live VA↔FRA private-network exercise  
- Third-party pen-test certification  
- Customer contracts, ARR, or logo list  
- Agent 2 storage / volume / DB migration features (explicitly out of scope)

## License one-liner

No use, copy, modify, distribute, sublicense, or production deploy until written commercial license **or** completed acquisition/asset transfer.

## Deal process (suggested)

NDA → run `cargo test --workspace` + evaluate scripts → review `acquisition/` → LOI → definitive agreement with counsel.

## Contact

[GitHub @theworker02](https://github.com/theworker02) · [Contact page](https://theworker02.github.io/cross-region-private-networking/contact.html)
