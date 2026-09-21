# Buyer FAQ — v1.1.0 (expanded)

## License & commercial

**Q: Can we run this in production under the public LICENSE?**  
A: No. Production use requires a written commercial license or completed acquisition/asset transfer. Evaluation only under written NDA if granted. Cloning ≠ license.

**Q: What happens to LICENSE after closing?**  
A: Deal agreements supersede LICENSE to the extent of conflict. See `legal-review/LICENSE_TRANSITION_NOTES.md` (**DRAFT — NOT EXECUTED**).

**Q: Can we open-source after acquisition?**  
A: Only if the acquirer owns the rights and chooses to; third-party crates remain under their licenses. Not automatic.

## Affiliation & positioning

**Q: Are you affiliated with Render?**  
A: No. Independent project. Messaging extends same-region private networking concepts; it does not claim partnership.

**Q: Are you competing with Render private networking?**  
A: No. Same-region private networking is treated as the preferred path (`LOCAL_NATIVE`). Fabric addresses cross-region / identity / policy gaps.

## Technical maturity

**Q: Is live VA↔FRA on Render proven in this package?**  
A: No. **NOT EXECUTED — INFRASTRUCTURE UNAVAILABLE**. SIMULATED drills are labeled. See `../docs/LIVE_MULTI_REGION.md`.

**Q: Is rustls mTLS real?**  
A: Yes — in-process Fabric CA + WebPkiClientVerifier + TrustStore post-check; covered by automated tests (`mtls::tests::*`).

**Q: Is there a third-party pen-test?**  
A: No. Internal assessment only (`SECURITY_PENTEST_INTERNAL.md`).

**Q: What about Agent 2 storage / volume migration?**  
A: Out of scope. Networking/fabric only.

## IP & legal drafts

**Q: What IP do we get?**  
A: Subject to definitive agreement — `IP_MANIFEST.md`, `ASSET_REGISTER.md`. OSS deps stay under upstream licenses.

**Q: Are patents included?**  
A: No filed patents are asserted in this package. Freedom-to-operate is buyer diligence.

**Q: Are legal docs in `legal-review/` signed?**  
A: No. All marked **DRAFT FOR PROFESSIONAL LEGAL REVIEW — NOT EXECUTED**.

**Q: Contributor assignments?**  
A: See `CONTRIBUTOR_RECORD.md` — counsel must complete before closing.

## Evaluation

**Q: How do we verify?**  
A: `EVALUATION_GUIDE.md`, `BUILD_REPRODUCIBILITY.md` — checkout tag, `cargo test --workspace`, evaluate scripts.

**Q: v1.0.0 vs v1.1.0?**  
A: v1.0.0 = frozen implementation baseline. v1.1.0 = expanded diligence/docs/site. Prefer reviewing both; implementation tag remains meaningful for code hash.

## Contact

**Q: How do we reach the Rightsholder?**  
A: GitHub [@theworker02](https://github.com/theworker02) or the [Contact page](https://theworker02.github.io/cross-region-private-networking/contact.html). No dedicated deal email is published in-repo.
