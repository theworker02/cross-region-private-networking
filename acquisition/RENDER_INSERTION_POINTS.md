# Render insertion points (conceptual, post-acquisition) — v1.1.0

**DRAFT product concept for acquirer eng/product — not a partnership commitment.**  
**Independent; not affiliated with Render.**

## Intent

If an acquirer (especially a platform with region-scoped private networking) absorbs this IP, where could fabric capabilities insert into the product surface?

## Conceptual insertion map

| Surface | Insertion idea | Notes |
|---------|----------------|-------|
| Private networking docs / UX | “Cross-region” tab explaining fabric vs native | Must praise same-region native path |
| Internal DNS / service discovery | Optional `*.global.internal` alongside private hostnames | Prefer native when same region |
| Dashboard networking | Fabric node status per region | Ops complexity — gated feature |
| mTLS / identity | Shared TrustStore with platform identity roadmap | Avoid duplicate PKI without plan |
| Policy | Namespace allowlists for cross-region | Default-deny |
| PrivateLink | “Complement” mode to reach PL-attached region | Document constraints |
| CLI / API | `resolve`, `doctor`, quarantine | Align with platform CLI later |

## What not to insert

- Hairpinning same-region traffic over public fabric underlay  
- Silent replacement of private hostnames  
- Storage / volume migration product lines (out of scope)  
- Trademark-confusing “official Render Fabric” branding without legal clearance  

## Organizational ownership (suggested)

| Owner | Responsibility |
|-------|----------------|
| Networking / edge | Dataplane, ingress, CIDRs |
| Identity / security | CA, TrustStore, policy |
| Developer experience | Naming, docs, LOCAL_NATIVE defaults |
| Legal | License transition, marks |

## Validation gates before GA (buyer)

1. Live multi-region exercise with labeled receipts  
2. External pen-test  
3. CIDR accuracy vs platform egress  
4. Customer preview with clear beta labeling  
