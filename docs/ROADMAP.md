# Roadmap

**Status:** Planning document for a vision-stage product. Dates are **not** commitments.  
**Related:** `PRODUCT.md`, `ARCHITECTURE.md`

---

## Principles

1. **Honesty over hype** — do not mark items “Done” without artifacts in the repo or a licensed distribution.  
2. **MVP proves the demo story** — VA→FRA via `payments.internal`, mTLS, policy, health.  
3. **Hardening before multi-tenant SaaS** — licensed single-tenant may precede any hosted offering.  
4. **Platform constraints first-class** — ephemeral disks, region networking, ingress shapes.

---

## Phase 0 — Documentation & IP package (current intent)

| Item | Notes |
|------|-------|
| Acquisition package | `ACQUISITION.md` |
| Proprietary license | `LICENSE` |
| Product / architecture / threat / demo docs | `docs/*` |
| Security & contribution policy | `SECURITY.md`, `CONTRIBUTING.md` |

**Exit:** Diligence-ready story without false claims of shipped code.

---

## Phase 1 — MVP fabric

**Goal:** Two-region encrypted fabric with internal naming and basic policy.

| Deliverable | Description |
|-------------|-------------|
| Fabric node binary/service | Dataplane + minimal control |
| Peer mTLS | Join + authenticated link |
| Service registration | Catalog for at least one remote service |
| Internal name resolution | `*.internal` or documented equivalent |
| Allow/deny policy | Service→service |
| Health checks | Backend probe + fail closed/predictably |
| Demo runbook | Align with `DEMO.md` |
| Basic metrics/logs | Link up, requests, errors |

**Exit:** Recorded or live demo matching Act III–V in `DEMO.md`.

---

## Phase 2 — Operability

| Deliverable | Description |
|-------------|-------------|
| Cert rotation | Overlap window; alerts on expiry |
| Multi-node readiness | Active-passive or HA sketch per region |
| GitOps-friendly config | Versioned policies |
| Runbooks | Incident: peer down, CA emergency |
| Secret management guide | Platform secrets; no keys in git |

**Exit:** Another operator can bring up fabric from docs alone.

---

## Phase 3 — Hardening & identity depth

| Deliverable | Description |
|-------------|-------------|
| Workload identity (optional) | Beyond node-only mTLS |
| Policy lint / expiry on wide allows | Prevent `allow *` footguns |
| Stronger admin RBAC | Separated admin plane |
| External audit readiness | Threat model refresh; pen-test if funded |
| Supply-chain basics | Lockfiles, image scanning when code exists |

---

## Phase 4 — Durable control plane & “persistent disk opportunity”

Render and similar platforms often use **ephemeral filesystems**. Local writes can vanish on deploy/restart.

| Opportunity | Why it matters |
|-------------|----------------|
| **Persistent disk** (or external DB/object store) for catalog, CA material, audit buffers | Survive restarts; enable warmer failover |
| Snapshotted policy history | Faster recovery / audit |
| Warm standby fabric node reading shared state | HA beyond “hope the container stayed up” |

This phase is explicitly called out as a **product opportunity**, not a claim that disks are already integrated.

---

## Phase 5 — Expansion (later)

| Idea | Notes |
|------|-------|
| Hub-and-spoke multi-region | Beyond pairwise VA↔FRA |
| Admin UI | Read-only first |
| Broader platform targets | Only after Render-shaped MVP is solid |
| Commercial licensed support tiers | Per `SUPPORT.md` |
| Hosted multi-tenant fabric SaaS | Requires legal + isolation design; **not** allowed for unlicensed parties under `LICENSE` |

---

## Explicitly deferred / out of scope for early phases

- Full Istio feature parity  
- Claiming official Render partnership  
- Patent filing strategy (track as `[PATENT_STATUS]` separately)  
- Guaranteed latency SLOs without measurement  

---

## Tracking

Use issues/milestones in a **private** tracker for non-public plans. Public repo should not leak confidential timelines or customer names (`TRADE_SECRETS.md`).

---

*© 2026 [Rightsholder]. Proprietary — see `LICENSE`.*
