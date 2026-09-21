# Demo script (SIMULATED) — v1.1.0

**License:** Proprietary — sale/acquisition only. Evaluation under written NDA if granted.  
**Full guide:** [`EVALUATION_GUIDE.md`](./EVALUATION_GUIDE.md)

```powershell
git checkout v1.1.0
cargo test --workspace
.\evaluate.ps1
cargo run -p fabric-cli -- resolve payments.internal
cargo run -p fabric-cli -- resolve payments.global.internal
cargo run -p fabric-cli -- doctor
cargo run -p fabric-cli -- benchmark
```

Browser SIMULATION: https://theworker02.github.io/cross-region-private-networking/demo.html  

Longer narrative: [`../docs/ACQUISITION_DEMO.md`](../docs/ACQUISITION_DEMO.md).

All multi-region path timings without live infra are labeled **SIMULATED**. Live multi-region: **NOT EXECUTED — INFRASTRUCTURE UNAVAILABLE**.
