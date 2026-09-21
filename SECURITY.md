# Security Policy

## Reporting a vulnerability

If you believe you have found a security vulnerability in this project’s materials, designs, or any published implementation:

1. **Do not** open a public GitHub issue for vulnerability details.  
2. **Do not** disclose exploit details publicly until coordinated disclosure is agreed.  
3. Email **`[SECURITY_EMAIL]`** (fallback: `[CONTACT_EMAIL]`) with:

   - Description of the issue and affected component (docs, design, or code if present)  
   - Steps to reproduce or a proof-of-concept **only** as needed to understand impact  
   - Your contact information and preferred disclosure timeline  
   - Whether you are reporting under an NDA or as an unsolicited researcher  

We aim to acknowledge reports within **`[SECURITY_SLA_ACK]`** (placeholder, e.g. 5 business days) and to provide a status update within **`[SECURITY_SLA_UPDATE]`** (placeholder, e.g. 14 business days). Timelines depend on Rightsholder availability for this pre-sale project.

## Scope

**In scope (examples):**

- Design flaws in the documented fabric architecture that would enable cross-tenant or cross-region unauthorized access *if implemented as specified*  
- Cryptographic protocol mistakes in published designs (mTLS, key rotation, identity)  
- Secrets accidentally committed to this repository  
- Misleading security claims in documentation that could cause unsafe deployments  

**Out of scope (examples):**

- Denial of service against third-party platforms (including Render)  
- Social engineering of individuals  
- Findings that require unauthorized access to systems you do not own  
- Issues solely in third-party dependencies without a clear, actionable interaction with this project’s designs  
- Theoretical issues with no realistic exploit path and no documentation/code hook  

## Coordinated disclosure

Please allow a reasonable window for assessment and mitigation before public disclosure. We prefer coordinated disclosure and will credit researchers who request it (unless you prefer anonymity), subject to not publishing confidential acquisition materials.

Public discussion of **high-level** classes of risk already covered in `docs/THREAT_MODEL.md` is fine; do not publish private diligence materials or unreleased implementation details.

## Unlicensed use is prohibited

This project is distributed under a **proprietary pre-sale license** (`LICENSE`). Discovering a vulnerability **does not** grant you rights to:

- Use, deploy, or operate the Product  
- Copy Materials for competitive purposes  
- Bypass access controls on private diligence data rooms  

Security research against **your own** licensed deployments (once commercially licensed) should follow the terms of that commercial agreement.

## Safe harbor (good faith)

The Rightsholder will not pursue legal action against individuals who:

- Act in good faith to identify and report vulnerabilities  
- Avoid privacy violations, data destruction, and service disruption  
- Do not exploit findings beyond what is necessary to demonstrate the issue  
- Comply with this policy and applicable law  

This safe-harbor statement is not a license to use the Product and is not legal advice.

## Security design references

- [`docs/THREAT_MODEL.md`](./docs/THREAT_MODEL.md)  
- [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md)  
- [`TRADE_SECRETS.md`](./TRADE_SECRETS.md)  

---

*© 2026 [Rightsholder]. Not legal advice.*
