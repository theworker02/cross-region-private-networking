# Buyer FAQ — v1.0.0

**Q: Can we run this in production today under the LICENSE?**  
A: No. Production use requires a written commercial license or completed acquisition/asset transfer. Evaluation only under written NDA if granted.

**Q: Are you affiliated with Render?**  
A: No. Independent project. Documentation describes how the fabric *extends* same-region private networking concepts.

**Q: Is live VA↔FRA on Render proven?**  
A: Not in this package. Live multi-region: **NOT EXECUTED — INFRASTRUCTURE UNAVAILABLE**. SIMULATED drills are labeled.

**Q: Is there a third-party pen-test?**  
A: No. See internal assessment `SECURITY_PENTEST_INTERNAL.md` and `SECURITY_REVIEW.md`.

**Q: What IP do we get?**  
A: Subject to definitive agreement — see `IP_MANIFEST.md` and `ASSET_REGISTER.md`. Third-party OSS remains under its licenses (`DEPENDENCY_LICENSES.md`, `SBOM.json`).

**Q: Are legal docs in `legal-review/` signed?**  
A: No. Every draft is marked **DRAFT FOR PROFESSIONAL LEGAL REVIEW — NOT EXECUTED**.

**Q: Agent 2 storage / volume migration?**  
A: Out of scope. This product is networking/fabric only.

**Q: How do we verify the build?**  
A: `BUILD_REPRODUCIBILITY.md` — `cargo test --workspace`, `evaluate.ps1` / `evaluate.sh`.
