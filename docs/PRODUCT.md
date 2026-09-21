# Product Specification — Cross-Region Private Networking

**Status:** v1.0.0 implementation frozen; v1.1.0 diligence package expanded — see `acquisition/` and `RELEASE_FREEZE.md`  
**Audience:** Buyers, licensees, future implementers  
**Related:** `ARCHITECTURE.md`, `THREAT_MODEL.md`, `DEMO.md`, `ROADMAP.md`, `../acquisition/PRODUCT_BRIEF.md`

---

## 1. Product summary

**Cross-Region Private Networking** is a Render-oriented **encrypted service fabric**. Fabric nodes attach to regional private networks and bridge them so workloads can reach multi-region dependencies using **internal names**, **mTLS**, **identity-aware policy**, and **health-aware routing** — without making every service publicly reachable as the default pattern.

**Primary job-to-be-done:**  
“Let my Virginia service call my Frankfurt service as `payments.internal` over a private, authenticated fabric path.”

---

## 2. Goals and non-goals

### Goals

| ID | Goal |
|----|------|
| G1 | Preserve **private addressing within each region**; only fabric nodes speak cross-region over encrypted links |
| G2 | Provide **stable internal DNS** (or DNS-compatible) names across the fabric |
| G3 | **Authenticate** fabric peers with mTLS; support identity rotation |
| G4 | Enforce **authorization policies** between services/namespaces |
| G5 | **Health-aware** routing and failover across peers/backends |
| G6 | Feel like a **service fabric**, not only a VPN (discovery + policy + naming) |
| G7 | Fit **Render** deployment shapes (services, private networking, multi-region) |

### Non-goals (MVP)

| ID | Non-goal |
|----|----------|
| N1 | Replace Render’s in-region private networking |
| N2 | Full Kubernetes service-mesh feature parity (Istio-class) |
| N3 | Guaranteed partnership with or endorsement by Render |
| N4 | Cross-cloud fabric as MVP (AWS↔GCP↔Render) — may be later |
| N5 | Being a consumer VPN or general-purpose SD-WAN product |

---

## 3. Personas

| Persona | Needs |
|---------|--------|
| **Platform / SRE** | Repeatable multi-region connectivity, ops runbooks, health, rotation |
| **Application owner** | Simple internal hostname, minimal public exposure |
| **Security / compliance** | Clear trust boundaries, mTLS, auditability, least privilege |
| **Buyer / corp-dev** | Defensible IP, realistic roadmap, diligence artifacts |

---

## 4. Core concepts

### 4.1 Fabric node

A deployable component (e.g. Render private service) that:

- Attaches to a **regional private network**
- Terminates **cross-region encrypted links** to peer fabric nodes
- Participates in **discovery**, **DNS**, **policy**, and **health**
- Optionally fronts or proxies selected east-west traffic for remote services

### 4.2 Fabric domain / mesh

A logical grouping of fabric nodes and registered services under a shared trust root (certificate authority or equivalent identity system). Example: `prod-fabric` spanning `oregon`, `virginia`, `frankfurt`.

### 4.3 Service registration

Workloads (or operators) register a service with:

- Name (e.g. `payments`)
- Region / private network membership
- Endpoints (private IPs/ports or Render private hostnames)
- Health check definition
- Optional identity / audience claims

### 4.4 Internal DNS

Clients resolve fabric names such as:

- `payments.internal`
- `payments.fra.fabric` (optional hierarchical scheme)

Resolution returns addresses that steer traffic to the **local fabric node** or an approved local path, which then forwards cross-region as needed.

*Exact DNS integration mechanism is an implementation choice* (see `ARCHITECTURE.md`): sidecar resolver, split-horizon DNS, hosts injection, or application SDK.

### 4.5 Identity and mTLS

- **Node-to-node:** mandatory mTLS on the cross-region bridge  
- **Workload-to-node (optional tiers):** mTLS or signed tokens for stronger identity  
- **Rotation:** short-lived certs; automated renew before expiry; revocation list or short TTL as primary control  

### 4.6 Policy

Declarative allow/deny rules, for example:

```text
allow: checkout.va → payments.fra
deny:  * → admin.fra   except role=platform
```

Policies should be versioned and auditable. MVP may be coarse (service→service); later identity attributes and rate limits.

### 4.7 Health-aware routing

- Active or passive health checks against registered endpoints  
- Remove unhealthy backends from rotation  
- Prefer healthy remote peer; fail closed or fail to documented degraded mode  
- Support drain for deploys  

---

## 5. User journeys

### 5.1 Operator brings up VA↔FRA fabric

1. Deploy `fabric-node` in VA (private network A).  
2. Deploy `fabric-node` in FRA (private network B).  
3. Bootstrap trust (share root CA / join token under secure channel).  
4. Nodes form mTLS peer link; health goes green.  
5. Register `payments` in FRA; verify VA resolves `payments.internal`.  
6. Attach policy allowing `checkout` → `payments`.  

### 5.2 Developer consumes a remote service

1. Configure app to call `https://payments.internal` (or HTTP cleartext only inside private network per policy — prefer TLS everywhere).  
2. No public URL required for the happy path.  
3. Observe latency/error metrics via fabric or app instrumentation.  

### 5.3 Certificate rotation

1. Control plane issues new leaf certs.  
2. Nodes overlap old/new during grace period.  
3. Old certs expire; alerts if rotation fails.  

---

## 6. Functional requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| F1 | Establish encrypted authenticated link between ≥2 regional fabric nodes | MVP |
| F2 | Register/deregister services with metadata | MVP |
| F3 | Resolve internal names to reachability paths | MVP |
| F4 | Enforce allow/deny policies on cross-region service access | MVP |
| F5 | Health checks influence routing | MVP |
| F6 | Cert/identity rotation without full mesh downtime | MVP |
| F7 | Basic metrics: link up/down, RTT, request counts/errors | MVP |
| F8 | Multi-node per region (HA) | Later |
| F9 | Fine-grained identity (SPIFFE-like IDs) | Later |
| F10 | Persistent-disk-backed control state / durable catalog opportunities | Later (see roadmap) |
| F11 | Admin UI / declarative GitOps configs | Later |
| F12 | Cross-account / multi-workspace fabrics | Later |

---

## 7. Non-functional requirements

| Area | Target (aspirational design goals — not measured SLOs yet) |
|------|--------------------------------------------------------------|
| Security | Encrypt cross-region; mutual auth; least privilege policies |
| Reliability | Prefer graceful degrade with clear errors over silent misroute |
| Operability | Logs with correlation IDs; documented runbooks |
| Performance | Minimize extra hops; avoid hairpinning when local |
| Portability | Primary: Render; designs should not needlessly lock to one secret store |
| Compliance narrative | Private path + identity + audit hooks — buyer validates for actual frameworks |

---

## 8. Configuration surface (illustrative)

```yaml
# illustrative only — not a shipped schema guarantee
fabric:
  name: prod
  trust_domain: fabric.example.internal
nodes:
  - id: va-1
    region: oregon  # example labels; use actual Render regions in impl
    listen: 0.0.0.0:8443
peers:
  - id: fra-1
    endpoint: fabric-fra.example.com:8443
services:
  - name: payments
    region: frankfurt
    backends:
      - address: payments:10000   # private service hostname:port
    health:
      path: /healthz
      interval: 5s
policies:
  - from: checkout
    to: payments
    action: allow
```

---

## 9. Integration with Render (constraints awareness)

Honest product constraints to design around:

- Private networking does not span regions — fabric exists **because** of this  
- Cross-region transit will traverse **public internet** (or other non-private paths) **between fabric nodes**  
- Outbound PrivateLink-style patterns toward AWS are **not** a substitute for Render↔Render private fabric  
- Platform limits (ports, timeouts, ephemeral disks, free-tier spin-down) affect control-plane durability — see roadmap note on **persistent disks** for durable state  

---

## 10. Packaging / delivery (future)

Possible delivery forms after implementation:

- Render Blueprint / IaC templates for dual-region fabric  
- Container image for fabric node  
- Documented runbooks and policy examples  
- Optional paid support under commercial license  

Until implemented, this repo delivers **specification and acquisition materials**, not a deployable product binary.

---

## 11. Success criteria (demo / MVP)

MVP is successful when:

1. VA workload reaches FRA service via internal name **without** exposing that service publicly as the primary path.  
2. Peer link refuses connections without valid client certs.  
3. Policy deny produces a clear failure.  
4. Killing the FRA backend flips health and stops routing (or fails predictably).  
5. Docs match observed behavior (`DEMO.md`).  

---

## 12. Open questions

- DNS mechanism of record for Render-native DX  
- Whether workload mTLS is required in MVP or node-to-node only  
- Control plane: embedded vs separate service  
- Multi-tenant SaaS offering vs single-tenant licensed software (commercial model)  

Record decisions in architecture ADRs when implementation begins.

---

*© 2026 [Rightsholder]. Proprietary — see `LICENSE`.*
