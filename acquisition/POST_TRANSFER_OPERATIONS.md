# Post-transfer operations — v1.0.0

## Day 0–7

1. Pin Rust toolchain; enable CI on buyer org.
2. Generate **new** Fabric CA and node identities; retire seller demo material.
3. Configure real egress CIDRs for ingress allowlists (do not use example ranges in production).
4. Deploy fabric nodes per region; keep `LOCAL_NATIVE` preference for in-region traffic.
5. Wire observability (logs/metrics) to buyer stack.
6. Re-run evaluate harness in buyer CI as a smoke gate.

## Day 30

- Threat-model review against buyer threat actors
- External pen-test recommended before broad production
- Confirm LICENSE / NOTICE headers match deal schedules

## Out of scope

Storage migration, DB cutover, filesystem sync (Agent 2).
