# Market and ICP — v1.1.0

**Honesty:** This section is strategic framing for diligence. It does **not** assert measured TAM/SAM, ARR, win rates, or named customers.

## Ideal customer profile (ICP)

| Attribute | Fit signal |
|-----------|------------|
| Platform | Multi-region deployments on Render (or similar region-scoped private nets) |
| Workload | Service-to-service APIs across regions (payments, identity, inventory, etc.) |
| Compliance posture | Prefers private paths, identity, audit over “public + JWT only” |
| Team | Platform / SRE with appetite for fabric nodes (not only app developers) |
| Non-fit | Single-region only; needs storage migration; wants turnkey managed SaaS with SLA today |

## Why Render as the acquisition / integration target

1. Documented **same-region private networking** is strong — worth extending, not replacing.  
2. Documented **region isolation** creates a clear cross-region gap.  
3. PrivateLink patterns are **complementary** (often outbound-to-AWS), not a substitute for Render↔Render private fabric.  
4. Acquirer synergy: platform that already owns the private-network DX can absorb fabric semantics.

See `RENDER_GAP_MATRIX.md`, `RENDER_INTEGRATION_PLAN.md`, `RENDER_INSERTION_POINTS.md`.

## Positioning (complementary, not competitive)

| Frame | Use |
|-------|-----|
| **Complement** | “Extends same-region private networking into a global identity-aware fabric.” |
| Avoid | “Render has no private networking.” |
| Avoid | “We are official Render partners” (unless separately true) |
| Underlay honesty | Cross-region fabric underlay ≠ Render private networking |

## Buyer segments (hypothetical)

1. **Render (or Render-like PaaS)** — deepest product fit for insertion.  
2. **Multi-region SaaS on Render** — would consume fabric as customer; may not buy IP.  
3. **Infrastructure / mesh vendors** — may value protocol/IP more than Render-specific DX.

No segment is claimed as an active pipeline in this package.

## Pricing / GTM

Intentionally omitted. Negotiate under NDA with counsel. Public LICENSE grants no commercial use by default.
