# Architecture — Cross-Region Private Networking

**Status:** Design overview for a vision-stage product  
**Related:** `PRODUCT.md`, `THREAT_MODEL.md`

---

## 1. Context

Render private networks are **region-scoped**. Workloads in Virginia (VA) cannot privately address workloads in Frankfurt (FRA) using in-region private networking alone. Cross-region communication commonly uses **public networking + application auth**.

This architecture introduces **fabric nodes** that:

- Remain on each region’s private network with the apps  
- Form an **encrypted, mutually authenticated bridge** across regions  
- Provide discovery, naming, policy, and health-aware forwarding  

```
┌─────────────────────────────┐          untrusted transit           ┌─────────────────────────────┐
│  Region VA (private net)    │         (public Internet)            │  Region FRA (private net)   │
│                             │                                      │                             │
│  checkout ──┐               │                                      │               ┌── payments  │
│  other apps ┼──► fabric-va ═╬════════ mTLS / encrypted link ═══════╬═ fabric-fra ◄─┤             │
│             │               │                                      │               └── other     │
│  DNS/discovery (local)      │                                      │  DNS/discovery (local)      │
└─────────────────────────────┘                                      └─────────────────────────────┘
```

---

## 2. Trust boundaries

| Boundary | Trust assumption |
|----------|------------------|
| **Inside regional private network** | Apps and fabric node share a private L3 island; still apply least privilege (compromise of one app ≠ trust all) |
| **Fabric node** | Trusted to terminate cross-region crypto, enforce policy, and not exfiltrate arbitrarily — harden like a security gateway |
| **Cross-region link** | Transit is **untrusted**; confidentiality and integrity via encryption + mTLS |
| **Identity / CA** | Root of trust; compromise is critical (protect keys, prefer HSM/KMS when available) |
| **Operator / control plane** | Admin actions can rewrite policy; authenticate and audit |
| **Public marketing / GitHub** | Untrusted for secrets; no production keys in repo |

---

## 3. Logical components

| Component | Responsibility |
|-----------|----------------|
| **Dataplane** | Proxy/forward cross-region service traffic; terminate TLS; apply routing |
| **Control / membership** | Peer join, service catalog, policy distribution |
| **Identity** | Issue/rotate node (and optional workload) credentials |
| **Discovery** | Service registration; membership gossip or centralized catalog |
| **DNS provider** | Answer internal names consistent with catalog |
| **Health** | Probe backends and peers; feed router |
| **Observability** | Metrics, structured logs, optional tracing headers |

MVP may colocate several of these in a single fabric-node process.

---

## 4. Topology options

### 4.1 Pairwise hub (two regions)

Simplest demo topology: VA ↔ FRA full mesh of size 2.

**Pros:** Easy to reason about, ideal for demo.  
**Cons:** Does not illustrate multi-region routing.

### 4.2 Full mesh (N regions)

Every fabric node peers with every other.

**Pros:** Low path stretch.  
**Cons:** O(N²) peer management; cert and policy fanout grows.

### 4.3 Hub-and-spoke

One region (or dedicated hub) aggregates spokes.

**Pros:** Simpler ops for many spokes.  
**Cons:** Hub is availability and latency chokepoint.

### 4.4 Recommended starting point

**Pairwise / small full mesh** for MVP; design control APIs so hub-and-spoke can be added without rewriting dataplane contracts.

---

## 5. Peer reachability design options

Cross-region peers must establish connectivity despite NAT, platform ingress rules, and dynamic addresses.

| Option | Description | Tradeoffs |
|--------|-------------|-----------|
| **A. Public listener on fabric node** | Fabric node exposes a **minimal** authenticated endpoint (mTLS-only) on a public URL/port | Clear reachability; must harden ruthlessly; not the same as exposing every app |
| **B. Rendezvous / broker** | Both sides dial out to a broker that matches sessions | Easier NAT; adds third party / component to trust |
| **C. Existing tunnel provider** | Use Cloudflare Tunnel or similar only as transport under fabric policy | Faster bootstrap; external dependency; different trust story |
| **D. Mutual dial + hole punching** | Aggressive NAT traversal | Fragile on restrictive platforms |

**Design preference for Render-shaped MVP:** Option **A** or **A+B hybrid** — fabric nodes may have a dedicated public ingress **solely for peer fabric control/data**, while application services remain private. Document clearly that “public fabric port ≠ public application.”

---

## 6. Data path (request flow)

Example: `checkout` (VA) → `payments.internal` (FRA)

1. `checkout` resolves `payments.internal` → address of **local fabric-va** (or VIP).  
2. `checkout` connects to fabric-va (ideally TLS).  
3. fabric-va authorizes policy `checkout → payments`.  
4. fabric-va selects healthy route via **fabric-fra**.  
5. Encrypted mTLS session carries the request (HTTP CONNECT, HTTP/2 proxy, or L4 tunnel — implementation choice).  
6. fabric-fra forwards to `payments` on FRA private network.  
7. Response returns on the same path.  

**Locality rule:** If target is local to the caller’s region, fabric should short-circuit (no cross-region hairpin).

---

## 7. Control path

```
Operator / CI
    │  apply config (GitOps or API)
    ▼
Identity + Policy + Catalog store
    │  distribute
    ▼
fabric nodes (watch / push)
```

**State durability:** Ephemeral filesystems (e.g. on some PaaS plans) lose local writes on restart. Prefer external durable store or **persistent disks** for catalog/CA material when moving beyond demos (`ROADMAP.md`).

---

## 8. Identity architecture (sketch)

```
Root CA (offline or KMS-backed)
   └── Intermediate CA (fabric control plane)
          ├── node cert: spiffe://fabric/ns/va/node/va-1
          ├── node cert: spiffe://fabric/ns/fra/node/fra-1
          └── optional workload certs
```

- Prefer **short-lived** leaves (hours/days) over multi-year static certs.  
- Bind peer SAN/URI to node ID; reject name mismatch.  
- Rotate intermediates on a planned cadence; document emergency revocation.

*SPIFFE-like URIs are illustrative — not a commitment to full SPIFFE/SPIRE stack in MVP.*

---

## 9. DNS / naming design options

| Option | Mechanism | DX | Ops complexity |
|--------|-----------|----|----------------|
| Split-horizon DNS | Fabric runs DNS on private IP | Familiar `*.internal` | Need clients use that resolver |
| Sidecar / init | Inject resolver or hosts | Strong control | More moving parts |
| Library/SDK | App resolves via API | Explicit | App changes |
| Local DNS forwarder | systemd-resolved / dnsmasq pattern | Nice on VMs | Less native on all PaaS shapes |

Pick one primary for MVP and document client setup in runbooks.

---

## 10. Failure modes (architecture view)

| Failure | Desired behavior |
|---------|------------------|
| Peer link down | Mark remote services unavailable; alert; do not blackhole silently if possible |
| Backend unhealthy | Remove from pool; 503 with clear reason |
| Policy deny | Explicit deny (403/policy error) |
| Cert expired | Fail closed; page operators |
| Split brain catalog | Prefer version vectors / single writer; avoid conflicting allows |

---

## 11. Deployment sketch on Render

Illustrative only:

- `fabric-node` as a **private service** with optional **web service** or exposed port reserved for peer mTLS  
- App services remain private; talk to fabric over private networking  
- Secrets (join tokens, CA keys) via platform env/secret store — never commit  
- Horizontal scale: multiple fabric nodes behind consistent hashing or active-passive later  

Bind HTTP listeners to `0.0.0.0:$PORT` where the platform requires it; separate internal dataplane ports as design allows.

---

## 12. Comparison to “just WireGuard”

WireGuard (or similar) can provide encrypted IP connectivity. This architecture additionally targets:

- Service catalog and **DNS**  
- **Policy** at service identity layer  
- **Health-aware** service routing  
- Render-oriented packaging and ops story  

WireGuard may still appear **under** the fabric as a transport implementation detail — the product value is the fabric control/DX layer.

---

## 13. Open architecture decisions (ADR candidates)

1. L7 proxy vs L4 tunnel as primary dataplane  
2. Peer reachability option A/B/C  
3. DNS option of record  
4. Catalog: Raft/etcd-like vs single primary vs gossip  
5. Workload identity in MVP vs node-only  

---

*© 2026 [Rightsholder]. Proprietary — see `LICENSE`. CONFIDENTIAL when shared in diligence beyond public repo.*
