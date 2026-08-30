---
id: TASK-99.6
title: Generate Trojan outbounds for sing-box 1.13
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:51'
labels:
  - sing-box
  - trojan
  - outbound
milestone: m-7
dependencies:
  - TASK-99.1
references:
  - 'https://sing-box.sagernet.org/configuration/outbound/trojan/'
parent_task_id: TASK-99
priority: high
ordinal: 74000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add Trojan managed-runtime generation with exact password, TLS, ALPN/SNI, and supported transport mappings. Reject profiles whose stored security or transport data cannot be represented faithfully.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Trojan address, port, and percent-decoded password are preserved
- [ ] #2 TLS, ALPN, SNI, certificate-insecure, and supported transport fields map through shared builders
- [ ] #3 Missing authentication or incomplete security data fails before launch
- [ ] #4 No Xray-specific streamSettings keys leak into sing-box JSON
- [ ] #5 Representative fixtures pass sing-box v1.13.21 check
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
