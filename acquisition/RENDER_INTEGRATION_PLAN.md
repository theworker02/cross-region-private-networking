# Render integration plan — v1.0.0

**Independent project; not affiliated with Render.**

## Thesis

Prefer Render **same-region private networking** (`LOCAL_NATIVE`) whenever source and destination share a region. Use fabric paths only for **cross-region** authenticated connectivity and global names.

## Artifacts

| Doc | Role |
|-----|------|
| [`../docs/RENDER_GAP_MATRIX.md`](../docs/RENDER_GAP_MATRIX.md) | Cited public gaps (region-scoped private net) |
| [`../docs/RENDER_INTEGRATION_ARCHITECTURE.md`](../docs/RENDER_INTEGRATION_ARCHITECTURE.md) | Adapter / connector architecture |
| [`../docs/LIVE_MULTI_REGION.md`](../docs/LIVE_MULTI_REGION.md) | Live exercise status |

## Phased rollout (buyer)

1. Single-region: fabric optional; validate naming + policy against native private hostnames  
2. Dual-region SIMULATED: run acquisition demo  
3. Dual-region LIVE: buyer provisions services + fabric nodes (not executed in this package)  
4. PrivateLink complement: outbound-to-AWS patterns remain complementary, not a substitute for Render↔Render private fabric  

## Non-goals

Claiming partnership, using Render trademarks for endorsement, or shipping Agent 2 storage features.
