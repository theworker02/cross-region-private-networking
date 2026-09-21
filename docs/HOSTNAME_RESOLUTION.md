# Hostname Resolution

Cross-region private hostname resolution maps fabric names to **fabric routes**
(node / region / transport), never to public application endpoints or ephemeral
instance IPs.

## Name forms

| Form | Example | Behavior |
|------|---------|----------|
| Global (Phase 4) | `payments.fabric.internal` | Best eligible instance (may be remote) |
| Region-qualified | `payments.frankfurt.fabric.internal` | Instances in that region only |
| Instance-qualified | `payments.node-17.fabric.internal` | Exact node |
| Legacy (Phase 2/3) | `payments.internal` | Same as global short form |

Canonical identity is the **service + fabric node cryptographic identity**, not a Render instance IP.

## Cross-region proof

Virginia client → Frankfurt-only `payments`:

```bash
cargo run -p fabric-cli -- resolve payments.internal
cargo run -p fabric-cli -- diagnose payments.internal
```

**Before fabric:** no registry entry → resolve fails (cross-region private IPs unreachable).  
**After fabric:** resolves to Frankfurt node via `PUBLIC_FABRIC_INGRESS_MTLS` — encrypted authenticated transit over public path. This is **not** Render-native private networking.

## Cache / staleness

- Positive TTL (default 30s)
- Invalidate on health failure / deregistration
- Cache hit re-checks registry; dead targets are not silently reused

## Related

- `crates/fabric-core/src/dns.rs`, `namespace.rs`
- Threat model: `docs/THREAT_MODEL.md`
