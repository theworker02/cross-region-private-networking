# Demo script (SIMULATED) — v1.0.0

**License:** Proprietary — sale/acquisition only. Evaluation under written NDA if granted.

```powershell
cargo test --workspace
.\evaluate.ps1
cargo run -p fabric-cli -- resolve payments.internal
cargo run -p fabric-cli -- resolve payments.global.internal
cargo run -p fabric-cli -- doctor
cargo run -p fabric-cli -- benchmark
```

Longer narrative: [`../docs/ACQUISITION_DEMO.md`](../docs/ACQUISITION_DEMO.md).

All multi-region path timings without live infra are labeled **SIMULATED**. Live multi-region: **NOT EXECUTED — INFRASTRUCTURE UNAVAILABLE**.
