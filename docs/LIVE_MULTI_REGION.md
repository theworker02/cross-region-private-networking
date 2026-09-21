# Live multi-region instructions — v1.0.0

## Status

**NOT EXECUTED — INFRASTRUCTURE UNAVAILABLE**

This release does **not** include a completed live VA↔FRA (or SIN) exercise on real Render private networks. SIMULATED drills and receipts are provided instead (`ACQUISITION_DEMO.md`, phase reports).

## If a buyer runs LIVE later

1. Provision Render services in ≥2 regions with private networking enabled in each.  
2. Deploy fabric nodes with reachability to regional private hostnames.  
3. Configure real egress CIDRs on ingress allowlists.  
4. Issue production Fabric CA / leaf certs (do not reuse demo CA).  
5. Verify `LOCAL_NATIVE` in-region and `FABRIC_*` cross-region with labeled receipts.  
6. Record results as **LIVE** (never relabel SIMULATED as LIVE).

## Required env (buyer-side)

- Render API credentials in buyer secrets store (never commit)  
- Network path for fabric-node public ingress if using `FABRIC_RELAYED`

No live credentials are shipped in this repository.
