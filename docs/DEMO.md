# Demo Script — Before / After Cross-Region Fabric

**Purpose:** Buyer-facing narrative and operator checklist for a VA ↔ FRA style demo.  
**Honesty:** Adapt region names to whatever Render regions you actually deploy. If software is not yet implemented, treat this as a **storyboard** for a future live demo or click-through.

---

## 0. Demo goals (what “good” looks like)

By the end, the audience should believe:

1. **Before:** Private networking alone does not give VA→FRA private addressing.  
2. **After:** `checkout` in VA calls **`payments.internal`** without making `payments` a general public API as the primary path.  
3. The fabric uses **mTLS between nodes**, **policy**, and **health-aware** behavior.  

Time box: **10–15 minutes** live, or **5 minutes** narrated video.

---

## 1. Cast of characters

| Component | Region | Role |
|-----------|--------|------|
| `checkout` | VA (e.g. Virginia) | Caller application |
| `payments` | FRA (e.g. Frankfurt) | Callee application |
| `fabric-va` | VA | Fabric node |
| `fabric-fra` | FRA | Fabric node |

Optional extras: a deny-listed `admin` service; a dashboard of link health.

---

## 2. Setup (pre-demo)

- [ ] Two Render regions / private networks provisioned  
- [ ] `payments` deployed private-only in FRA  
- [ ] `checkout` deployed in VA  
- [ ] Fabric nodes deployed and peered (mTLS green)  
- [ ] DNS/internal name `payments.internal` resolving from VA path  
- [ ] Policy: `checkout → payments` allow  
- [ ] Health endpoint on `payments` (`/healthz`)  
- [ ] Browser or terminal ready for curl/httpie  
- [ ] Backup slides if live network fails  

**Do not** paste real join tokens or CA keys into chat recordings.

---

## 3. Act I — Before (pain)

**Narration:**  
“Render private networks are region-scoped. Our checkout service in Virginia needs payments in Frankfurt. On private networking alone, Virginia cannot privately address Frankfurt.”

**Live actions:**

1. From a VA shell / `checkout` one-off: attempt private hostname of FRA `payments` (as documented today) — show failure or lack of private route.  
2. Show the “usual” workaround sketch: public URL + JWT/API key.  
3. Call out risks: public attack surface, key distribution, no shared discovery/health fabric.

**Optional visual:** Simple diagram with a red X between VA private net and FRA private net.

**Checkpoint question to audience:**  
“Would you rather every cross-region dependency be public-by-default?”

---

## 4. Act II — Fabric up

**Narration:**  
“We deploy a fabric node in each region. Nodes form an encrypted, mutually authenticated bridge. Applications stay on private networking; only the fabric speaks cross-region.”

**Live actions:**

1. Show `fabric-va` and `fabric-fra` health: peer **UP**, cert expiry healthy.  
2. Briefly show (redacted) that handshake requires client certificates — unauthorized connect fails.  
3. Register `payments` in the catalog (or show it already registered).  
4. Resolve `payments.internal` from VA context — highlight that resolution steers via the fabric path.

---

## 5. Act III — After (happy path)

**Narration:**  
“Checkout calls `payments.internal` as if it were local. The fabric authorizes, routes to Frankfurt, and returns the response.”

**Live actions:**

1. From `checkout` (VA):

   ```bash
   # illustrative
   curl -sS https://payments.internal/v1/ping
   ```

2. Show **200** / expected payload.  
3. Optionally show fabric metrics: request count increment, RTT VA↔FRA.  
4. Flip to logs: policy **allow**, upstream **payments**.

**One-liner for buyers:**  
“Before: VA cannot privately address FRA. After: `payments.internal`.”

---

## 6. Act IV — Policy deny (trust)

**Live actions:**

1. Apply temporary deny `checkout → payments` (or call from an unauthorized identity).  
2. Repeat curl — expect **403 / policy denied** (exact code per implementation).  
3. Restore allow.

**Narration:**  
“This is not only a tunnel. Authorization is part of the fabric.”

---

## 7. Act V — Health-aware routing

**Live actions:**

1. Break `payments` health (scale to zero, fail `/healthz`, or block port).  
2. Show fabric marking backend **unhealthy**.  
3. Curl from checkout — **predictable failure** (503), not a hang if avoidable.  
4. Restore backend; show recovery.

**Narration:**  
“Health-aware routing beats wishing your JWT layer also did failover.”

---

## 8. Act VI — Contrast with alternatives (talk track)

Keep this verbal; avoid disparagement:

| Alternative | One-line contrast |
|-------------|-------------------|
| Public + JWT | Works; expands public surface; weak fabric semantics |
| WireGuard DIY | Great crypto; you still build discovery/DNS/policy/ops |
| Cloudflare Tunnels | Different trust/ops model; not Render private-net fabric |
| Classic service mesh | Powerful; often K8s-heavy; not Render-region-island native |
| AWS PrivateLink | Excellent in AWS; Render-related PrivateLink patterns are typically **outbound to AWS**, not R2R private fabric |

---

## 9. Closing ask (acquisition / license)

- Point to `ACQUISITION.md` (CONFIDENTIAL package).  
- Clarify status: vision / early — diligence will verify code maturity.  
- CTA: NDA → technical deep-dive → LOI.  
- Contact: `[CONTACT_EMAIL]`.

---

## 10. Failure contingency

| Glitch | Fallback |
|--------|----------|
| Peer link down | Show recorded terminal capture; debug after |
| DNS not resolving | Use explicit fabric proxy URL as interim |
| Region outage | Switch to alternate region pair labels in narration |

---

## 11. Demo anti-patterns

- Do not claim production customers or SLAs you have not measured.  
- Do not show real secrets.  
- Do not imply Render partnership unless true.  
- Do not demo unlicensed third-party accounts without permission.  

---

*© 2026 [Rightsholder]. Proprietary — see `LICENSE`.*
