# Render Gap Matrix

**Thesis:** Render already provides excellent **same-region** private networking. This fabric **extends** that experience into a **global, identity-aware private fabric**. It does **not** claim Render lacks private networking.

**Independent project; not affiliated with Render.**

## Classification legend

| Class | Meaning |
|-------|---------|
| `RENDER_NATIVE` | Provided by Render today (public docs) |
| `FABRIC_PROVIDED` | Supplied by this fabric product |
| `OVERLAPPING` | Similar goals; different mechanisms — must not fight native path |
| `COMPLEMENTARY` | Fabric fills a documented gap next to Render primitives |

## Capability matrix

| Capability | Class | Notes | Sources |
|------------|-------|-------|---------|
| Same-region private network between workspace services | RENDER_NATIVE | Unique private hostnames; no public internet hop | [Private Network](https://render.com/docs/private-network) |
| Stable private hostnames / internal DB URLs (same region) | RENDER_NATIVE | | [Private Network](https://render.com/docs/private-network) |
| Separate private network **per region** | RENDER_NATIVE (constraint) | Regions are isolated private nets | [Regions](https://render.com/docs/regions), [Private Network](https://render.com/docs/private-network) |
| Direct cross-region private networking (Render↔Render) | FABRIC_PROVIDED | Render docs: cross-region needs secured public traversal | [Regions — Private networking](https://render.com/docs/regions) |
| AWS PrivateLink Render → AWS (same-region link creation) | RENDER_NATIVE | Pro+; initiate from Render to AWS VPC endpoint service | [Private Link](https://render.com/docs/private-link) |
| PrivateLink reverse (AWS → Render private) | COMPLEMENTARY gap | Docs: not reverse | [Private Link](https://render.com/docs/private-link) |
| Private DNS for PrivateLink endpoints | COMPLEMENTARY / FABRIC_PROVIDED aliasing | Docs: Private DNS not supported; use dashboard DNS name + TLS/SNI | [Private Link](https://render.com/docs/private-link) |
| Cross-region access to a PrivateLink attached in one region | FABRIC_PROVIDED (complement mode) | Fabric path to region that holds the PrivateLink | [Private Link](https://render.com/docs/private-link) (same-region PL constraint) |
| Change service/DB region in place | RENDER_NATIVE gap | Must create new + migrate | [Regions — Changing](https://render.com/docs/regions) |
| Workspace-level trust on private net (no zero-trust by default) | RENDER_NATIVE | App-layer auth recommended | [How Render handles private networking](https://render.com/articles/how-render-handles-private-networking.md) |
| Identity-aware service policy across regions | FABRIC_PROVIDED | Complements same-region private net | This repo |
| Global names (`*.global.internal`, `*.fabric.internal`) | FABRIC_PROVIDED | Prefer local native when same-region | This repo |
| Local-bypass (use Render private hostname in-region) | OVERLAPPING / COMPLEMENTARY | Fabric must not hairpin same-region via public underlay | This repo |

## Messaging rules

1. Praise Render same-region private networking.  
2. Position fabric as **global extension + identity layer**, not a replacement.  
3. When fabric peers use public ingress underlay, say so — never call that underlay “Render private networking.”  
4. PrivateLink complement mode only claims **publicly documented** PL capabilities.

## Cite checklist (operator diligence)

- https://render.com/docs/private-network  
- https://render.com/docs/regions  
- https://render.com/docs/private-link  
- https://render.com/changelog/securely-access-non-render-resources-over-aws-privatelink  
- https://render.com/articles/how-render-handles-private-networking.md  
