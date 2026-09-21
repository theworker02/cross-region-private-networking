# Render Integration Architecture

**Thesis:** Extend Render’s excellent **same-region** private networking into a **global identity-aware private fabric**. Independent project; not affiliated with Render.

## Public capabilities (pre-acquisition / licensed)

| Capability | Mechanism |
|------------|-----------|
| Same-region traffic | **LOCAL_NATIVE** — Render private hostname (no fabric hairpin) |
| Cross-region service names | `*.fabric.internal` / `*.global.internal` via fabric nodes |
| Peer auth | mTLS identity + admission + optional public fabric ingress allowlist |
| Policy | Identity ACL default-deny |
| PrivateLink complement | Remote region → fabric → region with documented PrivateLink → AWS resource |
| DNS aliases | Explicit cert/SNI policy; never auto-disable hostname verification |

## Post-acquisition internal (design only — not claiming access)

Deeper platform hooks (native sidecar injection, first-class private DNS for PL, reverse PL) would require Render engineering coordination. This package documents fit; it does **not** include non-public Render internals.

## Trust boundaries

See `docs/RENDER_GAP_MATRIX.md` and `docs/THREAT_MODEL.md`.

## Underlay honesty

Cross-region fabric peer links may traverse the public Internet (PUBLIC_FABRIC_INGRESS_MTLS). That underlay is **not** Render private networking.
