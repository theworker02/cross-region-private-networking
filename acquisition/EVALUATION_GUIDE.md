# Evaluation guide — v1.1.0

## Purpose

Give buyer engineers a repeatable path from clone → confidence without inventing LIVE multi-region results.

## Preconditions

- Rust stable toolchain  
- Network for crates.io (first build)  
- Windows PowerShell or Unix shell  

## Steps

```powershell
git clone https://github.com/theworker02/cross-region-private-networking.git
cd cross-region-private-networking
git checkout v1.1.0   # or v1.0.0 for code-freeze baseline comparison
cargo test --workspace
.\evaluate.ps1
# Unix: ./evaluate.sh
```

## Expected artifacts (`evaluation/`)

| File | Meaning |
|------|---------|
| `SUMMARY.md` | Version + test line + timestamps |
| `test_output.txt` | Full test log |
| `resolve_*.json` | Name resolution samples |
| `doctor.txt` | Local sanity |
| `benchmark.json` | SIMULATED timings |
| `evaluate_cli.json` | Bundle JSON |

## Pass / fail heuristics

| Check | Pass |
|-------|------|
| `cargo test --workspace` | All tests ok |
| Evaluate SUMMARY | Version matches checkout; tests line present |
| Resolve outputs | JSON produced; path classes sensible |
| Labels | No artifact claims LIVE multi-region unless buyer ran LIVE |

## Demo narrative

- CLI: `DEMO_SCRIPT.md`  
- Longer: [`../docs/ACQUISITION_DEMO.md`](../docs/ACQUISITION_DEMO.md)  
- Browser: https://theworker02.github.io/cross-region-private-networking/demo.html (**SIMULATION**)

## Optional deep dives

1. Read `SECURITY_REVIEW.md` statuses  
2. Skim `TECHNICAL_DUE_DILIGENCE.md` module map  
3. Walk `RENDER_GAP_MATRIX.md` citations  

## Contact for blocked evaluation

GitHub [@theworker02](https://github.com/theworker02)
