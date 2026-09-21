# Release Notes — v0.4.0

First numbered engineering release of the Cross-Region Private Networking fabric prototype.

## Highlights

- Resolve `payments.internal` / `payments.fabric.internal` from Virginia to Frankfurt-only instances via fabric routes
- Public fabric ingress controls: mutual TLS identity + configurable IP allowlist
- Network-only `fabric evacuate-region --dry-run`
- 56 automated tests passing locally
- Pages demo site (SIMULATION labeled)

## Honest gaps

- Live multi-region infrastructure: **NOT EXECUTED — INFRASTRUCTURE UNAVAILABLE**
- RTT/overhead numbers in default harness are **SIMULATED**
- Not an open-source grant; proprietary license unchanged

## Verify

```bash
cargo test --workspace
cargo run -p fabric-cli -- resolve payments.internal
```
