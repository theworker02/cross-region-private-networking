# Render gap matrix (acquisition copy) — v1.1.0

Canonical engineering doc: [`../docs/RENDER_GAP_MATRIX.md`](../docs/RENDER_GAP_MATRIX.md).

This file summarizes the same thesis for diligence readers.

**Thesis:** Render already provides excellent **same-region** private networking. This fabric **extends** that experience into a **global, identity-aware private fabric**. It does **not** claim Render lacks private networking.

**Independent project; not affiliated with Render.**

## Capability snapshot

| Capability | Class | Notes |
|------------|-------|-------|
| Same-region private network | RENDER_NATIVE | Unique private hostnames |
| Separate private net per region | RENDER_NATIVE (constraint) | Regions isolated |
| Direct cross-region private (Render↔Render) | FABRIC_PROVIDED | Public docs: secured public traversal pattern |
| AWS PrivateLink Render→AWS | RENDER_NATIVE | Pro+; same-region PL constraints apply |
| Identity-aware policy across regions | FABRIC_PROVIDED | Complements native private net |
| Global names + LOCAL_NATIVE bypass | FABRIC_PROVIDED / COMPLEMENTARY | Must not hairpin same-region via public underlay |

## Sources (public)

- https://render.com/docs/private-network  
- https://render.com/docs/regions  
- https://render.com/docs/private-link  
- https://render.com/articles/how-render-handles-private-networking.md  

Buyers should re-verify citations at diligence time — public docs change.
