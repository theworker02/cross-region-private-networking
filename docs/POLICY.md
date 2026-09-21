# Policy

Declarative lines: `allow|deny from <svc> to <svc> port … proto … priority …`. Default-deny. Simulator: `fabric policy test --from … --to … --port …`. Distribution: version/epoch/digest with CURRENT/STALE/INVALID/UNKNOWN — refuse INVALID/UNKNOWN enforcement.
