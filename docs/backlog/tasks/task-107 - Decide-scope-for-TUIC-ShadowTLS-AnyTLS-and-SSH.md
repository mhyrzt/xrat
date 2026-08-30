---
id: TASK-107
title: Decide scope for TUIC ShadowTLS AnyTLS and SSH
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:52'
labels:
  - sing-box
  - protocols
  - scope
milestone: m-7
dependencies:
  - TASK-98
references:
  - 'https://github.com/yarikov/kvn-tui'
  - 'https://sing-box.sagernet.org/configuration/outbound/'
priority: low
ordinal: 81000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make an explicit product and architecture decision for the four protocol families advertised by kvn-tui but absent from Xrat's Protocol model: TUIC, ShadowTLS, AnyTLS, and SSH. Determine whether each belongs in this milestone, a later protocol-import milestone, or remains unsupported, based on normalization and runtime-generation requirements.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each protocol has a documented support decision and rationale
- [ ] #2 Required share-link parser, normalized model, persistence, deduplication, output, and validation work is estimated separately
- [ ] #3 ShadowTLS chaining requirements are distinguished from single-outbound protocols
- [ ] #4 The sing-box support matrix does not imply support for excluded protocols
- [ ] #5 Any approved implementation work is represented by review-sized follow-up tasks before coding
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
