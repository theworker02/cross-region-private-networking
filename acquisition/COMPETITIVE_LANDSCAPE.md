# Competitive landscape — v1.1.0

**Frame:** Comparison is descriptive. No claim of superiority in production benchmarks vs named vendors. Independent project; not affiliated with Render.

## Options teams use today

| Approach | Strengths | Gaps vs this vision |
|----------|-----------|---------------------|
| **Public + JWT / API keys** | Simple; works now | Public attack surface; key sprawl; weak discovery / health fabric semantics |
| **WireGuard / DIY VPN overlays** | Strong encryption; IP reachability | Not service-centric; DNS/policy/ops often DIY; weak “Render-native” DX |
| **Cloudflare Tunnels / similar** | Proven tunnel ops | Different trust model; not a Render private-net fabric between regions |
| **Service meshes (Istio, Linkerd, …)** | Rich mTLS, policy, observability | Cluster/K8s-centric; heavy; not tailored to Render private-network islands |
| **AWS PrivateLink** | Private service connectivity in AWS | Different platform; Render PL patterns are typically outbound-to-AWS, not Render↔Render private fabric |
| **This product** | Render-shaped: names, LOCAL_NATIVE preference, identity, policy, cross-region fabric | Early relative to hyperscaler meshes; live multi-region not executed here |

## Differentiation thesis

Optimize for **multi-region private service connectivity next to region-scoped private networks**, with fabric semantics (discovery, mTLS, policy, quarantine) — not replace AWS networking, not claim to be Cloudflare, and not pretend a generic VPN is a service fabric.

## Build vs buy (for acquirer)

| Path | When |
|------|------|
| **Acquire this IP** | Want architecture + working Rust prototype + diligence corpus quickly |
| **Build in-house** | Have mesh talent; accept longer time-to-DX |
| **License commercially** | Want rights without asset purchase |

## Open competitive diligence questions

See `OPEN_QUESTIONS.md` (prior art, trademark clearance, platform ToS).
