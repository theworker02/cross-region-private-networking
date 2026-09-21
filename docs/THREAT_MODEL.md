# Threat Model — Cross-Region Private Networking

**Method:** STRIDE-oriented analysis against the fabric architecture  
**Status:** Living document for a vision-stage design (not a formal audit)  
**Related:** `ARCHITECTURE.md`, `SECURITY.md`, `PRODUCT.md`

---

## 1. Assets to protect

| Asset | Why it matters |
|-------|----------------|
| Cross-region application payloads | May include PII, secrets, payment-adjacent data |
| Service credentials / tokens | Impersonation risk |
| Fabric node identity keys / CA | Mesh-wide compromise if stolen |
| Policy store | Attacker could authorize lateral movement |
| Service catalog / DNS answers | Poisoning → traffic interception |
| Management plane credentials | Full fabric takeover |
| Peer public endpoints | Attack surface on the internet |

---

## 2. Actors

| Actor | Intent / capability |
|-------|---------------------|
| Internet attacker | Scan/exploit public fabric listener |
| Malicious or compromised app in private network | Pivot via fabric to remote region |
| Compromised fabric node | Read/modify cross-region traffic; alter routing |
| Malicious peer (failed join auth) | Attempt to join mesh |
| Insider operator | Misconfigure allow-all policies |
| Curious unlicensed party | Misuse public docs/code if any |

---

## 3. Entry points

- Public peer port / URL on fabric nodes  
- Private network APIs of fabric (DNS, proxy, admin)  
- Config/CI pipelines that push policy  
- Supply chain (base images, dependencies) — when code exists  
- Social engineering of operators during diligence or ops  

---

## 4. STRIDE analysis

### 4.1 Spoofing

| Threat | Mitigation direction |
|--------|----------------------|
| Fake fabric peer | mTLS with pinned trust domain; reject unknown CAs |
| Fake service registration | Authenticate registrars; signed registrations; admin approval mode |
| DNS spoof on private net | Prefer authenticated discovery; limit who can run DNS; network controls |
| App spoofing identity | Optional workload mTLS / identity tokens; do not trust `From:` headers alone |

### 4.2 Tampering

| Threat | Mitigation direction |
|--------|----------------------|
| Modify packets in transit | Encrypted authenticated tunnels (TLS 1.2+ / modern AEAD) |
| Tamper policy in transit | Integrity-protected distribution; authenticate control plane |
| Bit-flip / replay | TLS anti-replay; avoid custom crypto |

### 4.3 Repudiation

| Threat | Mitigation direction |
|--------|----------------------|
| Admin denies dangerous policy change | Audit logs for policy/cert/admin actions; immutable log sink when possible |
| Lateral movement without trail | Log allow/deny decisions with request IDs |

### 4.4 Information disclosure

| Threat | Mitigation direction |
|--------|----------------------|
| Eavesdrop cross-region | Encrypt peer link; disable cleartext peer mode in prod |
| Catalog leaks internal topology | Restrict catalog API; treat topology as sensitive |
| Logs contain secrets | Redaction; never log Authorization headers or private keys |
| Public repo leaks | No secrets in git; `TRADE_SECRETS.md` handling |

### 4.5 Denial of service

| Threat | Mitigation direction |
|--------|----------------------|
| SYN/flood public peer port | Platform DDoS protections; rate limits; fail closed under load with alerts |
| Catalog explosion | Quotas on registrations |
| Health-check amplification | Bound probe rates; auth probes where applicable |
| Cert rotation outage | Overlap windows; monitoring on expiry |

### 4.6 Elevation of privilege

| Threat | Mitigation direction |
|--------|----------------------|
| App gains cross-region access beyond policy | Default deny; explicit allows; continuous testing of deny paths |
| Compromised low-priv app talks to fabric admin API | Separate admin listener; mTLS + RBAC; not on shared app ports |
| Join token theft | Single-use/short-lived join tokens; rotate on suspicion |

---

## 5. Trust boundary deep dive

### 5.1 Compromised application (VA)

**Risk:** Uses fabric as a proxy to scan FRA services.  
**Controls:** Fine-grained policy; per-service identity; anomaly detection on destination fanout; network policies where available.

### 5.2 Compromised fabric node

**Risk:** Highest blast radius — can often see cross-region traffic and alter routing.  
**Controls:** Harden host; minimal privileges; separate admin identity; encrypt disks for key material; rapid revoke/reissue; prefer multiple nodes with detection of divergent behavior (later).

### 5.3 Public peer endpoint

**Risk:** Internet-facing attack surface.  
**Controls:** mTLS-only; no version banners; restrict cipher suites; optional IP allowlists for known peer egress; intrusion detection; keep apps off this port.

### 5.4 CA compromise

**Risk:** Attacker issues arbitrary node certs.  
**Controls:** Offline root; HSM/KMS; monitoring for unexpected issuances; emergency CRL/short TTL strategy.

---

## 6. Abuse cases (narrative)

1. **Shadow mesh:** Attacker deploys rogue “fabric” and tricks operators into joining — mitigate with out-of-band fingerprint verification of trust root.  
2. **Policy sprawl:** `allow * → *` for “temporary debug” left in prod — mitigate with policy linting and expiry on wide allows.  
3. **DNS poisoning:** Malicious colocated process answers `*.internal` — mitigate by locking resolver configuration and authenticating discovery.  
4. **Competitive misuse of docs:** Unlicensed party implements competing service from diligence docs — mitigate with `LICENSE`, NDA, and trade-secret process (legal, not technical).  

---

## 7. Cryptography notes (design intent)

- Prefer well-known libraries over custom protocols.  
- TLS for peer links; avoid inventing record layers.  
- Document cipher policy when implementation exists.  
- Key material never in application logs or public repositories.  

---

## 8. Residual risks (accepted for early stage)

- Platform provider compromise (Render or underlying cloud) is largely out of product control.  
- Side-channel / advanced persistent threats not fully modeled.  
- No claim of formal verification or third-party audit until one is performed.  
- Documentation-only repo cannot be “penetration tested” as a running system until deployed.  

---

## 9. Security verification checklist (when implemented)

- [x] Peer without cert cannot complete handshake (unit tests in `handshake`, `ingress`)  
- [x] Expired identity fails closed (`identity` tests)  
- [x] Policy deny works under automation tests (`policy` tests)  
- [x] Public fabric ingress rejects missing mTLS and non-allowlisted IPs (`ingress` tests)  
- [ ] Secrets scanners clean on CI (not wired in this release)  
- [ ] Rotation drill documented and rehearsed (API present; ops drill pending)  

Report vulnerabilities per [`SECURITY.md`](../SECURITY.md).

---

## 10. Who can join the fabric (implemented controls)

**Claim scope:** only what `fabric-core` actually enforces in code as of v0.4.0.

### 10.1 Admission path

| Step | Control | Code |
|------|---------|------|
| 1 | Generate cryptographic node identity (Ed25519), not IP-based | `identity::NodeIdentity` |
| 2 | Join request → `PENDING` | `admission::AdmissionController` |
| 3 | Operator authorize → `AUTHORIZED` | same |
| 4 | Successful handshake → `ACTIVE` | handshake + admission |
| 5 | Revoke → `REVOKED`; cannot re-authorize without new epoch / new process | admission + trust CRL |

**Config alone is insufficient.** Possessing YAML/env without an admitted verifying key does not grant mesh membership.

### 10.2 Public fabric ingress

Optional transport: **PUBLIC_FABRIC_INGRESS_MTLS** (encrypted authenticated transit over the Internet).

| Check | Behavior |
|-------|----------|
| Client certificate / mapped `PublicIdentity` | Required; missing → reject |
| TrustStore authenticate (known, not revoked, not expired) | Required |
| Source IP in allowlist | Required when allowlist configured (`deny_when_empty`) |
| Labeling | Never called “private network” |

**CIDR limitations (honest):** Render outbound IP ranges are **shared across tenants** in a region. An allowlist of those CIDRs reduces arbitrary Internet scanning but is **not** unique tenant identity. Pair with mTLS + admission. Sample CIDRs in-repo are **EXAMPLE/TEST ONLY** — operators must load current ranges from [Render docs](https://render.com/docs) / dashboard via `config/render_outbound_cidrs.json` or `FABRIC_IP_ALLOWLIST`.

### 10.3 Threat scenarios covered by tests

| Scenario | Outcome |
|----------|---------|
| Unknown peer handshake | Reject `UnknownIdentity` |
| Replayed handshake nonce | Reject `ReplayedHandshake` |
| Revoked identity rejoin by address | Reject |
| Valid cert + IP outside allowlist | Reject |
| Allowlisted IP without mTLS | Reject |
| Stale revoked membership gossip | No resurrection (`membership` tests) |

### 10.4 Not claimed

- Full TLS stack termination in production (logic is identity/allowlist gate + session KDF; integrate with platform TLS terminators)  
- Formal proof of handshake  
- Live multi-region pen-test results  

---

*© 2026 [Rightsholder]. Not a certification or warranty. Proprietary — see `LICENSE`.*
