# Release Notes — v0.5.0

Acquisition-candidate release positioning fabric as a **global extension** of Render same-region private networking.

## Verify

```powershell
cargo test --workspace
.\evaluate.ps1
cargo run -p fabric-cli -- evaluate
```

## Artifacts

- `PHASE5_REPORT.md`
- `ACQUISITION_READINESS_REPORT.md`
- `evaluation/SUMMARY.md` (after evaluate)
- `acquisition/SBOM.json`, `TRANSFER_INVENTORY.json`

## Checksums

Generate after packaging:

```powershell
Get-FileHash PHASE5_REPORT.md,ACQUISITION_READINESS_REPORT.md,Cargo.toml -Algorithm SHA256
```

Live multi-region: **NOT EXECUTED — INFRASTRUCTURE UNAVAILABLE**.
