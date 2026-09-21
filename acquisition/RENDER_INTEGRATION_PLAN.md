# Render integration plan — v1.1.0 (expanded)

**Independent project; not affiliated with Render.**

## Goals

1. Never fight Render same-region private networking — prefer `LOCAL_NATIVE`.  
2. Provide authenticated cross-region connectivity and global names.  
3. Keep underlay labeling honest (fabric public ingress ≠ Render private net).  
4. Remain adapter-shaped so platform APIs can evolve.

## Reference artifacts

| Doc | Role |
|-----|------|
| [`RENDER_GAP_MATRIX.md`](./RENDER_GAP_MATRIX.md) / [`../docs/RENDER_GAP_MATRIX.md`](../docs/RENDER_GAP_MATRIX.md) | Cited gaps |
| [`../docs/RENDER_INTEGRATION_ARCHITECTURE.md`](../docs/RENDER_INTEGRATION_ARCHITECTURE.md) | Connectors / adapters |
| [`RENDER_INSERTION_POINTS.md`](./RENDER_INSERTION_POINTS.md) | Post-acquisition product insertion |
| [`../docs/LIVE_MULTI_REGION.md`](../docs/LIVE_MULTI_REGION.md) | Live exercise status |

## Phased rollout (buyer)

### Phase A — Single region (validation)

- Deploy fabric node optional; validate naming + policy against native private hostnames.  
- Assert `LOCAL_NATIVE` for same-region resolves.  
- No cross-region claims required.

### Phase B — Dual region SIMULATED

- Run acquisition demo / evaluate harness.  
- Label all receipts **SIMULATED**.

### Phase C — Dual region LIVE (buyer-executed)

- Buyer provisions services in ≥2 regions.  
- Real CIDRs on ingress allowlists; production CA.  
- Record results as **LIVE** only when actually executed.  
- **Not executed in this package.**

### Phase D — PrivateLink complement

- Treat PrivateLink as complementary egress/ingress to AWS patterns.  
- Do not claim reverse PL or private DNS unless docs change.

## Control-plane integration sketch

1. **Discovery export** — register service instances with region + health.  
2. **Resolve API** — `payments.global.internal` → path class + endpoint.  
3. **Policy compile** — default-deny snapshots.  
4. **Admission** — TrustStore + mTLS at fabric ingress.  
5. **Observability hooks** — structured events (see `schemas/`).

## Failure modes to design for

| Failure | Expected fabric behavior |
|---------|--------------------------|
| Peer revoke | Deny handshake / ingress |
| Region drain | Evacuate network routes; no storage moves |
| Underlay partition | Mark FABRIC path unhealthy; do not silently use public app ports without policy |
| Stale route ads | Reject untrusted advertisements |

## Success criteria (buyer)

- ≥95% same-region resolves classified LOCAL_NATIVE in tests  
- Cross-region requires mutual auth  
- Docs/runbooks never call fabric underlay “Render private networking”
