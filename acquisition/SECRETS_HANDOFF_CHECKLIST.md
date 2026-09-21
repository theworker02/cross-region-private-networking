# Secrets handoff checklist — v1.0.0

**Principle:** This repository must not contain live secrets. Rotate anything that ever touched seller environments before or at closing.

- [ ] Confirm no `.env`, PEM private keys, or API tokens committed (`git grep` / secret scanners)
- [ ] Rotate GitHub tokens / deploy keys used for this repo
- [ ] Rotate any Render / cloud API keys used during evaluation
- [ ] Invalidate Fabric CA material used only in seller demos; buyer issues new CA
- [ ] Revoke NDA data-room credentials after exclusivity window
- [ ] Update FUNDING.yml / sponsorship links if buyer does not want seller handles
- [ ] Deliver secrets (if any) **out of band** via approved escrow — never in git

If a secret is found in history: treat as incident, rotate, consider history rewrite only with counsel and coordinated force-push policy (not done in this release).
