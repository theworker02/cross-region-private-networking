# Build reproducibility — v1.0.0

## Toolchain

- Rust stable (edition 2021 workspace)
- Cargo workspace members: `fabric-core`, `fabric-cli`

## Reproduce

```bash
git checkout v1.0.0
cargo test --workspace
./evaluate.sh          # or evaluate.ps1 on Windows
cargo run -p fabric-cli -- doctor
```

## Expected

- All workspace tests pass
- `evaluation/SUMMARY.md` written with version **1.0.0**
- CLI resolve/doctor/benchmark JSON artifacts under `evaluation/`

## Non-goals

- Bit-identical binaries across OS/arch without pinned toolchain file (buyer may add `rust-toolchain.toml`)
- Reproducing LIVE multi-region results (none executed)

## Integrity

See `CHECKSUMS.txt` for how to hash release trees. Tag `v1.0.0` is the authoritative source snapshot.
