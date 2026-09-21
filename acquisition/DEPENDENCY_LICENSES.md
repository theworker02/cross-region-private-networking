# Dependency licenses — v1.0.0

Direct workspace dependencies (from `Cargo.toml`). Re-verify with `cargo license` / `cargo-deny` before distribution.

| Crate | Typical license | Class |
|-------|-----------------|-------|
| serde / serde_json | MIT OR Apache-2.0 | PERMISSIVE |
| thiserror / anyhow | MIT OR Apache-2.0 | PERMISSIVE |
| ed25519-dalek / x25519-dalek | BSD-3-Clause | PERMISSIVE |
| rand / rand_core | MIT OR Apache-2.0 | PERMISSIVE |
| sha2 / hkdf / hex | MIT OR Apache-2.0 | PERMISSIVE |
| chrono | MIT OR Apache-2.0 | PERMISSIVE |
| clap | MIT OR Apache-2.0 | PERMISSIVE |
| uuid | Apache-2.0 OR MIT | PERMISSIVE |
| parking_lot | MIT OR Apache-2.0 | PERMISSIVE |
| tracing / tracing-subscriber | MIT | PERMISSIVE |
| rustls | Apache-2.0 OR ISC OR MIT | PERMISSIVE |
| rustls-pki-types | Apache-2.0 OR ISC OR MIT | PERMISSIVE |
| rcgen | MIT OR Apache-2.0 | PERMISSIVE |
| tokio | MIT | PERMISSIVE |
| criterion / tempfile (dev) | Apache-2.0 OR MIT / MIT OR Apache-2.0 | PERMISSIVE (dev) |
| fabric-core / fabric-cli | LicenseRef-Proprietary | OWNED |

**Copyleft:** none identified among direct deps.  
**UNKNOWN:** none — see `SBOM.json`.

Full notices: `THIRD_PARTY_NOTICES.md` (this folder) and root `../THIRD_PARTY_NOTICES.md`.
