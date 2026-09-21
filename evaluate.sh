#!/usr/bin/env bash
# One-command acquisition evaluation (Unix).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"
mkdir -p evaluation fuzz/crashes

echo "== fabric evaluate ==" | tee evaluation/SUMMARY.md
echo "version: 1.1.0" | tee -a evaluation/SUMMARY.md
echo "started: $(date -u +%Y-%m-%dT%H:%M:%SZ)" | tee -a evaluation/SUMMARY.md

cargo test --workspace 2>&1 | tee evaluation/test_output.txt
PASS_LINE=$(grep -E "test result: ok\." evaluation/test_output.txt | tail -1 || true)
echo "tests: $PASS_LINE" | tee -a evaluation/SUMMARY.md

cargo run -q -p fabric-cli -- resolve payments.internal > evaluation/resolve_payments.json 2>evaluation/resolve_err.txt || true
cargo run -q -p fabric-cli -- resolve payments.global.internal > evaluation/resolve_global.json 2>>evaluation/resolve_err.txt || true
cargo run -q -p fabric-cli -- doctor > evaluation/doctor.txt 2>>evaluation/resolve_err.txt || true
cargo run -q -p fabric-cli -- benchmark > evaluation/benchmark.json 2>>evaluation/resolve_err.txt || true
cargo run -q -p fabric-cli -- evaluate > evaluation/evaluate_cli.json 2>>evaluation/resolve_err.txt || true

echo "artifacts: evaluation/*.txt evaluation/*.json" | tee -a evaluation/SUMMARY.md
echo "label: SIMULATED where noted; live multi-region NOT EXECUTED" | tee -a evaluation/SUMMARY.md
echo "done: $(date -u +%Y-%m-%dT%H:%M:%SZ)" | tee -a evaluation/SUMMARY.md
