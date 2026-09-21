# Trade Secrets and Confidentiality

**Classification guidance for this repository and related diligence materials.**

Copyright (c) 2026 [Rightsholder]. See `LICENSE` and `NOTICE.md`.

---

## 1. Purpose

This document explains what the Rightsholder treats as **confidential / trade secret** information, how it is marked, and expectations for NDAs during evaluation, licensing, and acquisition.

It does **not** replace a signed NDA. Where this document and an NDA conflict, the **signed NDA controls**.

---

## 2. What is confidential

Unless the Rightsholder has **explicitly** published something as public marketing, treat the following as confidential:

| Category | Examples |
|----------|----------|
| **Acquisition & commercial** | Pricing discussions, deal structures under negotiation, LOI drafts, buyer lists, diligence Q&A |
| **Technical unpublished detail** | Non-public protocol choices, key hierarchy details beyond high-level docs, internal runbooks, unpublished roadmaps with dates/commitments |
| **Security-sensitive** | Private vulnerability reports, incident details, private keys, credentials, data-room access methods |
| **Business** | Conversations with potential customers or partners, cost models, exclusivity terms |
| **Derivatives** | Notes, extracts, and models derived from confidential Materials |

High-level public README positioning may be shareable; **detailed architecture options, threat-model exploit paths, and acquisition economics** are not for unconstrained redistribution.

---

## 3. What may be less sensitive (still proprietary)

The following may appear in the public repo but remain **proprietary** (not open source):

- Vision-level product descriptions  
- Competitive comparisons at a high level  
- The existence of a pre-sale process  

Proprietary ≠ public domain. `LICENSE` still prohibits use, copying for productization, and competitive reverse engineering.

---

## 4. Marking

Preferred markings:

- Documents: header or title containing **`CONFIDENTIAL`** (see `ACQUISITION.md`)  
- Emails / data rooms: label **Confidential — Subject to NDA**  
- Oral disclosures: state confidentiality at the start of diligence calls when sharing non-public detail  

Failure to mark a particular file does **not** waive trade-secret protection if the information is not generally known and is subject to reasonable secrecy efforts (including this policy and `LICENSE`).

---

## 5. NDA expectations

Before sharing non-public diligence materials, the Rightsholder typically requires:

1. A written **mutual or one-way NDA**  
2. Identification of the receiving party’s deal team  
3. Purpose limitation (evaluation of license/acquisition only)  
4. No reverse engineering for competing products  
5. Return or destruction of Materials on request or at process end  
6. Survival of confidentiality obligations for an agreed term  

Evaluation under NDA is also contemplated in `LICENSE` §3.1.

---

## 6. Handling rules for recipients

If you receive confidential Materials:

- Limit access to need-to-know personnel under confidentiality duties  
- Do not upload to public AI training corpora, public tickets, or unrestricted Slack/Discord channels  
- Do not fork/publish confidential extracts to public GitHub  
- Use secure channels for credentials; prefer Rightsholder-issued time-limited access  
- Report suspected unauthorized disclosure to `[SECURITY_EMAIL]` / `[CONTACT_EMAIL]` promptly  

---

## 7. Employees, contractors, and contributors

Anyone who creates Materials for this project should have written **IP assignment** to the Rightsholder. Unsolicited public contributions are governed by `CONTRIBUTING.md` and should not introduce third-party confidential information.

---

## 8. Duration

Trade-secret protection lasts as long as information remains secret and valuable. Contractual NDA terms may specify a fixed survival period for contractual confidentiality (often independent of trade-secret status).

---

## 9. Not legal advice

Trade-secret law varies by jurisdiction. Counsel should tailor NDAs, legends, and enforcement strategy for real transactions.

---

*Last updated: 2026-03-21 · Related: `ACQUISITION.md`, `LICENSE`, `NOTICE.md`*
