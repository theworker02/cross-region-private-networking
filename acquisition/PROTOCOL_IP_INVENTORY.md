# Protocol / original IP inventory — v1.1.0

**Purpose:** Help counsel and eng inventory **original** designs vs commodity crypto/standards.

**Honesty:** Using TLS, DNS concepts, Ed25519, etc. does not create exclusive IP in those standards. Originality claims below are about **composition and product-specific protocol choices** in this repo — not patent grants.

## Original / product-specific components (asserted as Materials)

| Component | Description | Evidence |
|-----------|-------------|----------|
| Path classification | `LOCAL_NATIVE` / `FABRIC_DIRECT` / `FABRIC_RELAYED` with in-region preference | `docs/`, routing / global_dns |
| Global naming scheme | `*.global.internal` resolve semantics preferring native | `global_dns`, hostname docs |
| Fabric CA + TrustStore post-check | Leaf SPIFFE-style URI embeds node id; TrustStore after rustls | `mtls.rs` |
| Route advertisement security | Reject untrusted / impersonating ads | `route_security` |
| Quarantine forensics | Remove routes but retain forensics | `quarantine` |
| Policy compiler + differential tests | Snapshot + randomized agreement | `policy_compiler` |
| Network-only evacuate | Explicitly non-storage | `evacuate`, drills |
| Render-shaped connector model | LOCAL_NATIVE bypass vs fabric underlay labeling | connectors / gap matrix |

## Commodity / third-party (not “sold” as exclusive IP)

| Component | Notes |
|-----------|-------|
| rustls / rcgen / ring | Upstream licenses — SBOM |
| ed25519-dalek / x25519-dalek | Upstream |
| tokio, serde, clap, … | Upstream |
| TLS 1.2/1.3, X.509, DNS concepts | Standards |

## Patent posture

`[PATENT_STATUS]`: **No filed patents asserted in this package.** Buyer FTO diligence recommended.

## Related

`IP_MANIFEST.md`, `PROTOCOL_IP_INVENTORY.md` (this file), root `TRADE_SECRETS.md`.
