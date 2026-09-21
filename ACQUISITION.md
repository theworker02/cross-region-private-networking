# CONFIDENTIAL — Acquisition One-Pager Package

**Product (working title):** Cross-Region Private Networking / Render-native encrypted service fabric  
**Diligence package:** 1.1.0 (expanded data room)  
**Implementation baseline:** v1.0.0 frozen  
**Status:** Acquisition-ready implementation + expanded diligence room — see `ACQUISITION_READINESS_REPORT.md`  
**Classification:** CONFIDENTIAL — for qualified buyers under NDA only  
**Copyright:** © 2026 [Rightsholder — e.g. theworker02 / legal entity name]  
**Independence:** Independent project; **not affiliated with Render.**
**Contact:** [GitHub @theworker02](https://github.com/theworker02)

> **Notice:** Provided for acquisition and commercial-license discussions. **No production use** under root `LICENSE` until a written commercial license or completed acquisition/asset transfer. Not investment advice. Do not redistribute outside your deal team without written permission. See `LICENSE`, `TRADE_SECRETS.md`, `NOTICE.md`, and `acquisition/`.

---

## 1. Executive summary

Render’s private networking is **region-scoped**. Services in one region generally cannot privately address services in another region over Render’s internal network. The documented path for cross-region communication is typically **public networking plus application-level authentication**.

This product concept is a **Render-native encrypted service fabric**: fabric nodes that bridge regional private networks with discovery, mutual TLS (mTLS), internal DNS naming, identity-aware auth, health-aware routing, and policy controls — designed to feel closer to a **smart service fabric** than a raw VPN tunnel.

**What a buyer gets:** concept IP, architecture and threat models, product specifications, demo narrative, diligence materials, and (as developed) any associated code, configs, and operational artifacts — subject to a definitive agreement.

**What a buyer should assume today:** **v1.0.0** freezes the acquisition-candidate implementation. **v1.1.0** expands the diligence data room, docs, and site. SIMULATED multi-region drills and disclosed gaps remain (live multi-region not executed; no external pen-test). Do not treat this package as evidence of production customers, filed patents, or recurring revenue unless separately verified in diligence.

**Navigate the data room:** start at [`acquisition/README.md`](./acquisition/README.md) (orientation → market → integration → technical DD → IP → transfer → legal drafts).

---

## 2. The problem

| Constraint | Implication |
|------------|-------------|
| Private networks are region-scoped on Render | `svc` in Virginia cannot privately resolve/reach `svc` in Frankfurt over Render private networking alone |
| Official cross-region pattern | Expose over public network + authenticate at the app (API keys, JWT, mTLS at the edge, etc.) |
| Compliance-sensitive workloads | Teams often prefer private paths, controlled identity, and auditability over “public + hope the JWT is perfect” |
| DIY overlays | Operators reinvent WireGuard meshes, tunnels, or mesh sidecars — operational burden, inconsistent DX, weak Render-native integration |

**Pain in one sentence:** Multi-region Render customers need private, identity-aware, health-aware connectivity between regions without turning every service into a public endpoint.

---

## 3. Solution / product vision

An **encrypted fabric** of lightweight **fabric nodes** (one or more per region / private network island) that:

1. **Bridge** regional private networks over encrypted cross-region links (over the public internet or approved transit, terminated at fabric nodes — not by exposing every app publicly).
2. **Discover** services and publish stable **internal DNS** names (e.g. `payments.internal`).
3. **Authenticate** peer fabric nodes and workloads with **mTLS** and rotatable identities.
4. **Authorize** with policies (which services/namespaces may talk; optional identity claims).
5. **Route** with health awareness and failover when a peer or backend is degraded.
6. Remain **smarter than a VPN**: service-centric naming, policy, discovery, and operational controls — not only IP reachability.

**Positioning:** Render-native private *service fabric* for cross-region communication — not a generic cloud VPN product, and not a claim of partnership with Render unless separately established.

---

## 4. Demo story (buyer narrative)

### Before

- Service `checkout` in **Virginia (VA)** needs `payments` in **Frankfurt (FRA)**.
- Private DNS / private networking does not provide a private VA→FRA path.
- Team exposes `payments` publicly (or via ad-hoc tunnel) and adds JWT/API-key checks.
- Failure modes: accidental public exposure, key sprawl, inconsistent health routing, hard-to-audit east-west paths.

### After (target experience)

- Fabric nodes in VA and FRA establish an encrypted, authenticated link.
- `payments` is registered in the fabric; VA workloads call **`payments.internal`** (or equivalent internal name).
- Traffic stays on private networks inside each region; only fabric nodes speak cross-region over the encrypted bridge.
- Health-aware routing and mTLS reduce “public + app auth only” as the default cross-region pattern.

**Demo script detail:** see `docs/DEMO.md`.

---

## 5. Market / ICP

**Ideal customer profile (ICP):**

| Segment | Why they care |
|---------|----------------|
| Multi-region Render deployments | Need VA↔EU (or similar) service calls without public-by-default |
| Compliance-sensitive teams | Prefer private paths, identity, audit logs, clear trust boundaries |
| Platform / SRE teams on Render | Want a reusable fabric vs one-off tunnels per service pair |
| Product companies expanding regions | Latency, residency, or DR patterns that still need internal APIs |

**Non-goals for ICP messaging (honesty):** This package does not claim current ARR, logo customers, or market-share data. Market size should be validated by the buyer’s own research during diligence.

---

## 6. Competitive landscape

| Approach | Fit | Gaps vs this vision |
|----------|-----|---------------------|
| **DIY public + JWT / API keys** | Simple; works today | Public attack surface; key/JWT ops; weak service discovery/health fabric semantics |
| **WireGuard / custom VPN overlays** | Strong encryption & IP reachability | Not service-centric; DNS/discovery/policy/ops often DIY; less “Render-native” DX |
| **Cloudflare Tunnels (and similar)** | Proven egress/ingress tunnels | Different trust/ops model; not a Render private-network fabric between regions |
| **Service meshes (Istio, Linkerd, etc.)** | Rich mTLS, policy, observability | Cluster/K8s-centric; heavy; not tailored to Render private networking region islands |
| **AWS PrivateLink** | Private service connectivity in AWS | Different platform; **Render’s PrivateLink-related patterns are typically outbound-to-AWS**, not Render-region-to-Render-region private fabric |
| **This product concept** | Render-shaped fabric: discovery, mTLS, internal DNS, identity, health-aware routing across regions | Early / vision-stage; buyer must diligence implementation maturity |

**Differentiation thesis:** Optimize for **Render multi-region private service connectivity** with fabric semantics — not replace AWS networking, not claim to be Cloudflare, and not pretend a generic VPN is a service fabric.

---

## 7. Technical architecture overview

High level (see `docs/ARCHITECTURE.md` for detail):

```
[VA private network]                    [FRA private network]
  apps ──► fabric-node-va ═══ mTLS/encrypted bridge ═══ fabric-node-fra ◄── apps
              │                                              │
         discovery /                                      discovery /
         internal DNS                                     internal DNS
```

**Core planned components:**

- Fabric node dataplane + control plane hooks
- Service discovery & catalog
- Internal DNS (or DNS-compatible naming)
- mTLS identity issuance & rotation
- Policy engine (allow/deny, identity-aware)
- Health checks & failover routing
- Observability hooks (logs/metrics — buyer may integrate with existing stacks)

**Trust boundaries:** Apps trust their regional fabric node; fabric nodes mutually authenticate; cross-region link is encrypted; public internet is untrusted transit.

---

## 8. Intellectual property & assets included in a sale

Subject to definitive agreement schedules, a typical asset package may include:

| Asset class | Examples |
|-------------|----------|
| **Documentation IP** | This repo’s product, architecture, threat model, demo, roadmap |
| **Design IP** | Topology options, naming schemes, policy models, rotation designs |
| **Software (as developed)** | Source, configs, IaC sketches, prototypes — *only what exists at closing* |
| **Brand / naming** | Working titles and marks as assigned (trademarks if any — confirm in diligence) |
| **Know-how** | Runbooks, demo scripts, diligence Q&A |
| **Excluded (unless listed)** | Seller personal accounts, unrelated repos, third-party licensed components (see `THIRD_PARTY_NOTICES.md`) |

**Honesty note:** Do not assume patents are filed. Patent status should be confirmed in diligence (`[PATENT_STATUS]` placeholder).

---

## 9. Suggested deal structures

| Structure | When it fits | Notes |
|-----------|--------------|-------|
| **Asset sale** | Buyer wants IP + materials outright | Cleanest for “buy the concept and build in-house”; schedules must list included/excluded assets |
| **Exclusive license** | Seller retains shell entity; buyer gets exclusive field-of-use | Define field (e.g. Render-related private fabric), territory, term, sublicense rights |
| **Non-exclusive commercial license** | Seller may continue other activities | Lower exclusivity premium; clear non-compete carve-outs |
| **Acqui-hire elements** | Buyer wants people + IP | Optional consulting / employment term sheets; not assumed |
| **Staged option / LOI → APA** | Early diligence | Option fee + exclusivity window common; counsel to draft |

Commercial terms (price, earnouts, escrows) are **intentionally omitted** here — negotiate under NDA with counsel.

---

## 10. Diligence checklist (buyer)

Use as a starting list; expand with counsel:

- [ ] Confirm Rightsholder legal identity and authority to sell/license
- [ ] Inventory of all Materials (repo, private docs, demos, credentials — no secrets in transfer without rotation plan)
- [ ] Implementation maturity: vision vs prototype vs production-ready
- [ ] Third-party OSS / cloud ToS constraints (`THIRD_PARTY_NOTICES.md`)
- [ ] Trademark / domain / naming conflicts
- [ ] Patent / prior art posture (`[PATENT_STATUS]`)
- [ ] Security: threat model review (`docs/THREAT_MODEL.md`), any prior incidents
- [ ] Dependency on Render platform APIs/behaviors (change risk)
- [ ] Employment / contractor IP assignment chain for any contributors
- [ ] Pending claims, disputes, or public disclosures of confidential material
- [ ] Transition assistance scope (docs only vs build support)
- [ ] Post-close LICENSE supersession and public repo disposition (archive/private/transfer)

---

## 11. Trade secrets & confidentiality

Detailed marking and handling rules: **`TRADE_SECRETS.md`**.

Summary: architecture choices, unpublished protocols, customer conversations, pricing discussions, and non-public demos are confidential. Public marketing claims must not leak unpublished implementation detail.

---

## 12. Contact / process (placeholders)

| Item | Placeholder |
|------|-------------|
| Primary contact | `[CONTACT_NAME]` |
| Email | `[CONTACT_EMAIL]` |
| Preferred process | NDA → teaser/one-pager → diligence data room → LOI → definitive agreement |
| Counsel | `[SELLER_COUNSEL]` |
| Response SLA (aspirational) | `[RESPONSE_SLA]` e.g. 2 business days |

**Inbound:** Qualified strategic buyers, platforms, and infrastructure companies exploring Render-adjacent networking. Please do not treat public GitHub issues as a deal channel for confidential negotiation.

---

## 13. Related documents in this repository

| Document | Purpose |
|----------|---------|
| `LICENSE` | Proprietary — no use/copy/modify/distribute/sublicense/production deploy until commercial license or acquisition |
| `acquisition/` | Expanded diligence data room (v1.1.0) — start at `acquisition/README.md` |
| `acquisition/PRODUCT_BRIEF.md` | Product brief for acquirer eng/product |
| `acquisition/MARKET_AND_ICP.md` | ICP / complementary framing |
| `acquisition/EVALUATION_GUIDE.md` | How to verify |
| `RELEASE_FREEZE.md` | v1.0.0 freeze notice |
| `RELEASE_NOTES_v1.0.0.md` / `RELEASE_NOTES_v1.1.0.md` | Release notes |
| `README.md` | Product overview & status |
| `SECURITY.md` | Vulnerability reporting |
| `NOTICE.md` | Copyright & trademarks |
| `TRADE_SECRETS.md` | Confidentiality handling |
| `CONTRIBUTING.md` | No public contribution license |
| `docs/PRODUCT.md` | Product specification |
| `docs/ARCHITECTURE.md` | Technical topology |
| `docs/THREAT_MODEL.md` | Security threat model |
| `docs/DEMO.md` | Demo script |
| `docs/ROADMAP.md` | MVP → later |

---

## 14. Disclaimer

This package is **CONFIDENTIAL** and informational. It does not create a binding offer. All figures, timelines, and capabilities are subject to verification. **Not legal, tax, or investment advice** — engage counsel for any transaction.

---

*Document version: 1.1.0 / 2026-09-21 · Classification: CONFIDENTIAL*
