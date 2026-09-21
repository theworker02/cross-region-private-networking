# Changelog

## [1.0.0] — 2026-09-21

**FROZEN** acquisition candidate — see [`RELEASE_FREEZE.md`](./RELEASE_FREEZE.md). Subsequent work should be a new version.

### Added

- Production-oriented **rustls mTLS** ingress (Fabric CA, WebPkiClientVerifier, TrustStore post-handshake)
- Complete acquisition diligence tree under `acquisition/` (IP, SBOM, transfer, security, legal drafts)
- Internal security assessment (`SECURITY_PENTEST_INTERNAL.md`) — not a third-party pen-test
- Docs: QUICKSTART, CLI, SECURITY, LIVE_MULTI_REGION; polished GitHub Pages site (demo, about, Render target, contact, due diligence)
- `RELEASE_NOTES_v1.0.0.md`, `RELEASE_FREEZE.md`; readiness report updated for 1.0

### Changed

- Authoritative version **1.0.0** (supersedes 0.5.0 as current release)
- LICENSE clarified: **no grant** until written commercial license **or** completed acquisition/asset transfer
- README / site / acquisition messaging aligned to sale-or-acquisition-only license stance

### Security

- Valid client mTLS accept/reject (missing cert, revoked identity) covered by workspace tests
- CIDR allowlist remains shared-control; must combine with mTLS + TrustStore

### Known limitations (disclosed)

- Live multi-region Render infrastructure exercise: **NOT EXECUTED — INFRASTRUCTURE UNAVAILABLE**
- No external/third-party penetration test
- Example CIDRs are not official Render ranges
- Legal drafts are **DRAFT FOR PROFESSIONAL LEGAL REVIEW — NOT EXECUTED**

## [0.5.0] — 2026-09-21

### Added

- Render gap matrix with cited public docs; integration architecture
- LOCAL_NATIVE / FABRIC_DIRECT / FABRIC_RELAYED path classification
- `*.global.internal` resolution (native first, else fabric)
- ExternalNetworkConnector + LocalMockConnector; PrivateLink complement plan
- Versioned control-plane API helpers; event JSON schema
- Policy compiler snapshots + differential/randomized testing
- Route advertisement security; `fabric quarantine`
- Regional + PrivateLink complement drills with receipts
- `evaluate.ps1` / `evaluate.sh` → `evaluation/`
- Fuzz corpus persistence under `fuzz/crashes/`
- Perf regression gates; SBOM; transfer inventory; legal-review drafts
- Pages: home, demo, render-fit, architecture, security, benchmarks, failure-lab, due-diligence
- Acquisition demo script; SECURITY_REVIEW; PROTOCOL_IP_INVENTORY

### Changed

- Version **0.5.0**; messaging centers extending Render private networking (not denying it)

### Security

- Adversarial route ads rejected; quarantine preserves forensics

### Known limitations

- Live multi-region NOT EXECUTED
- Ingress TLS terminator was OPEN at 0.5.0 (closed in 1.0.0 via rustls)
- Example CIDRs not official Render ranges

## [0.4.0] — 2026-09-21

Initial fabric prototype (identity, DNS, ingress, reports). See `RELEASE_NOTES_v0.4.0.md`.
