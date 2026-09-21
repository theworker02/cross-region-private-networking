# Quick start — v1.0.0

## Prerequisites

- Rust toolchain (stable, edition 2021)
- Windows PowerShell or Unix shell

## Build & test

```powershell
cargo test --workspace
```

## Evaluate (acquisition smoke)

```powershell
.\evaluate.ps1
# or: ./evaluate.sh
```

Artifacts: `evaluation/SUMMARY.md`, resolve/doctor/benchmark JSON.

## Resolve a name

```powershell
cargo run -p fabric-cli -- resolve payments.internal
cargo run -p fabric-cli -- resolve payments.global.internal
cargo run -p fabric-cli -- doctor
```

## License reminder

Proprietary — evaluation under written NDA only if granted; no production deploy until commercial license or acquisition. See [`../LICENSE`](../LICENSE).

## Next

- [`CLI.md`](./CLI.md) · [`ARCHITECTURE.md`](./ARCHITECTURE.md) · [`ACQUISITION_DEMO.md`](./ACQUISITION_DEMO.md)
