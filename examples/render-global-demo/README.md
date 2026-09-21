# Render Global Demo (SIMULATION)

One command (from repo root):

```bash
cargo run -p fabric-cli -- demo
```

## What this demonstrates (SIMULATED)

1. Virginia fabric node boots with cryptographic identity  
2. Frankfurt peer admitted + mTLS handshake  
3. Frankfurt-only `payments` registered  
4. Policy `checkout → payments` activated  
5. `payments.internal` / `payments.fabric.internal` resolves cross-region  
6. Transport labeled `PUBLIC_FABRIC_INGRESS_MTLS` (not private networking)  
7. Doctor shows evidence  
8. Optional: `cargo run -p fabric-cli -- evacuate-region frankfurt --dry-run`  
9. Optional: `cargo run -p fabric-cli -- benchmark` (SIMULATED overhead)  
10. Singapore (`sin`) can be added as a third region in custom sims via `fabric sim`

## Regions in story

| Label | Role |
|-------|------|
| Virginia (VA) | Caller / checkout |
| Frankfurt (FRA) | payments origin |
| Singapore (SIN) | Optional third region in large-fabric SIMULATED topology |

**NOT EXECUTED — INFRASTRUCTURE UNAVAILABLE** for live Render multi-region deploy in this package.
