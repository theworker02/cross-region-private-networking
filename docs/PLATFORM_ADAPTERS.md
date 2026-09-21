# Platform Adapters

`FabricCore` is platform-agnostic. Adapters implement `PlatformHooks`:

| Adapter | Notes |
|---------|-------|
| `LocalAdapter` | Dev data dir, `127.0.0.1` bind |
| `RenderAdapter` | `0.0.0.0:$PORT`, ephemeral FS flag, allowlist path `config/render_outbound_cidrs.json` |
| `FuturePlatformStub` | railway / fly / aws / gcp / azure / k8s placeholders |

## Render adapter V2 metadata (documented fields)

Intended metadata only — **no HTML scraping**:

- region  
- service ID  
- instance ID  
- discovery hostname  
- environment (production/staging)  
- internal hostname  

Populate from Render env vars / API tokens under operator control. FabricCore must not import Render dashboard HTML.

## Connectivity layering

1. DIRECT (authenticated)  
2. PUBLIC_FABRIC_INGRESS_MTLS (+ allowlist)  
3. AUTHORIZED_RELAY  

Never label (2) or (3) as private networking.
