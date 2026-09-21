# Product brief (for acquirer product / eng) — v1.1.0

## Product name (working)

**Cross-Region Private Networking** / **Fabric**

## Problem statement

Platforms that isolate private networking **per region** leave multi-region service meshes with a gap: same-region private hostnames work well; cross-region traffic often becomes public + app auth. Operators reinvent tunnels, meshes, or ad-hoc gateways with inconsistent DX and weak service-centric policy.

## Solution

A lightweight **service fabric** that:

1. Resolves stable internal names (`*.internal`, `*.global.internal`)
2. Prefers **native private hostnames** when source and destination share a region (`LOCAL_NATIVE`)
3. Uses **authenticated fabric paths** (`FABRIC_DIRECT` / `FABRIC_RELAYED`) across regions
4. Enforces **identity** (Ed25519 TrustStore + rustls mTLS) and **policy** (default-deny)
5. Supports **quarantine**, route advertisement security, and network-only evacuate drills

## Non-goals (v1.x)

- Replacing same-region private networking  
- Claiming affiliation with Render  
- Volume / filesystem / database migration (Agent 2)  
- Guaranteeing live multi-region production SLO without buyer validation  

## Primary personas

| Persona | Job to be done |
|---------|----------------|
| Platform eng | Cross-region private DX without DIY VPN mesh |
| Security / compliance | Identity + auditability on east-west paths |
| Corp-dev / product | Acquire complementary IP for multi-region private story |

## Success metrics (buyer-defined after close)

Examples (not claimed as current seller metrics): time-to-resolve global name; % in-region traffic staying LOCAL_NATIVE; mTLS reject rate for untrusted peers; MTTR for quarantine.

## Demo

SIMULATION Pages demo + CLI evaluate — see `DEMO_SCRIPT.md`, `EVALUATION_GUIDE.md`. Live multi-region: not executed in this package.
