# One-command acquisition evaluation (Windows PowerShell).
$ErrorActionPreference = "Continue"
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $Root
New-Item -ItemType Directory -Force -Path evaluation, fuzz/crashes | Out-Null

$summary = New-Object System.Collections.Generic.List[string]
$summary.Add("== fabric evaluate ==")
$summary.Add("version: 1.1.0")
$summary.Add("started: $((Get-Date).ToUniversalTime().ToString('o'))")

Write-Host "Running cargo test..."
cargo test --workspace 2>&1 | Tee-Object -FilePath evaluation/test_output.txt | Out-Null
$pass = Select-String -Path evaluation/test_output.txt -Pattern "test result: ok\. \d+ passed" |
  Where-Object { $_.Line -match "ok\. ([1-9]\d*) passed" } |
  Select-Object -Last 1
if ($pass) { $summary.Add("tests: $($pass.Line)") } else { $summary.Add("tests: see evaluation/test_output.txt") }

cargo run -q -p fabric-cli -- resolve payments.internal 2>$null | Out-File evaluation/resolve_payments.json -Encoding utf8
cargo run -q -p fabric-cli -- resolve payments.global.internal 2>$null | Out-File evaluation/resolve_global.json -Encoding utf8
cargo run -q -p fabric-cli -- doctor 2>$null | Out-File evaluation/doctor.txt -Encoding utf8
cargo run -q -p fabric-cli -- benchmark 2>$null | Out-File evaluation/benchmark.json -Encoding utf8
cargo run -q -p fabric-cli -- evaluate 2>$null | Out-File evaluation/evaluate_cli.json -Encoding utf8

$summary.Add("artifacts: evaluation/*")
$summary.Add("label: SIMULATED where noted; live multi-region NOT EXECUTED - INFRASTRUCTURE UNAVAILABLE")
$summary.Add("done: $((Get-Date).ToUniversalTime().ToString('o'))")
$summary | Out-File evaluation/SUMMARY.md -Encoding utf8
Write-Host ($summary -join [Environment]::NewLine)
