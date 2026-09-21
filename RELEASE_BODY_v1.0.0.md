## Cross-Region Private Networking — v1.0.0

**Acquisition candidate (FROZEN)** · Independent project; **not affiliated with Render**

### Thesis

Render’s private networking is excellent **within a region**. This release extends that model with global names, rustls mTLS identity, path classification (`LOCAL_NATIVE` / `FABRIC_*`), and policy for cross-region service connectivity.

### Highlights

- Rust workspace: `fabric-core` + `fabric-cli` at **1.0.0**
- Global hostname resolution with in-region **LOCAL_NATIVE** preference
- **rustls mTLS** ingress (Fabric CA + WebPki + TrustStore)
- Policy, quarantine, route advertisement security, network-only evacuate drills
- Complete **`acquisition/`** diligence package (IP, SBOM, transfer, security, legal drafts)
- GitHub Pages: [live site](https://theworker02.github.io/cross-region-private-networking/) · [SIMULATION demo](https://theworker02.github.io/cross-region-private-networking/demo.html) · [Target: Render](https://theworker02.github.io/cross-region-private-networking/render.html) · [Contact](https://theworker02.github.io/cross-region-private-networking/contact.html)
- Evaluate: `cargo test --workspace` · `evaluate.ps1` / `evaluate.sh`

### License

**Proprietary — sale / acquisition only.** No right to use, copy, modify, distribute, sublicense, or deploy in production until a **written commercial license** or **completed acquisition/asset transfer**. See [LICENSE](https://github.com/theworker02/cross-region-private-networking/blob/v1.0.0/LICENSE).

### Acquisition package

- [ACQUISITION_READINESS_REPORT.md](https://github.com/theworker02/cross-region-private-networking/blob/v1.0.0/ACQUISITION_READINESS_REPORT.md) — **READY_WITH_DISCLOSED_ITEMS**
- [acquisition/](https://github.com/theworker02/cross-region-private-networking/tree/v1.0.0/acquisition)
- [RELEASE_FREEZE.md](https://github.com/theworker02/cross-region-private-networking/blob/v1.0.0/RELEASE_FREEZE.md)

### Honest disclosures

1. Live multi-region Render exercise: **NOT EXECUTED — INFRASTRUCTURE UNAVAILABLE**
2. Internal security assessment only — **not** a third-party pen-test
3. Legal drafts: **DRAFT FOR PROFESSIONAL LEGAL REVIEW — NOT EXECUTED**
4. Example CIDRs are not official Render ranges

### Verify

```bash
git checkout v1.0.0
cargo test --workspace
```

### Contact

GitHub [@theworker02](https://github.com/theworker02) · [Contact page](https://theworker02.github.io/cross-region-private-networking/contact.html)
