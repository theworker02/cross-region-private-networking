# CLI reference — v1.0.0

Binary: `fabric` (`crates/fabric-cli`)

```text
cargo run -p fabric-cli -- <COMMAND>
```

## Common commands

| Command | Purpose |
|---------|---------|
| `resolve <name>` | Resolve `*.internal` / `*.global.internal` with path class |
| `doctor` | Local health / config sanity |
| `benchmark` | Honest SIMULATED timings → JSON |
| `evaluate` | Bundle evaluation JSON |
| `quarantine` | Quarantine controls (see `--help`) |

Exact flags: `cargo run -p fabric-cli -- --help` and `… <command> --help`.

## Labels

Outputs may include **SIMULATED** markers. Live multi-region is **NOT EXECUTED** in this package.
