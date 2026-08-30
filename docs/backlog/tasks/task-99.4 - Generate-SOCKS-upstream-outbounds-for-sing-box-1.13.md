---
id: TASK-99.4
title: Generate SOCKS upstream outbounds for sing-box 1.13
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:51'
labels:
  - sing-box
  - socks
  - outbound
milestone: m-7
dependencies:
  - TASK-98
references:
  - 'https://sing-box.sagernet.org/configuration/outbound/socks/'
parent_task_id: TASK-99
priority: medium
ordinal: 72000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add managed sing-box SOCKS upstream generation from normalized SOCKS5 imports. Preserve authentication and network behavior, and explicitly constrain support to versions represented by Xrat's current model.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 SOCKS5 address, port, username, and password map to documented sing-box fields
- [ ] #2 Unauthenticated and username/password profiles are both covered
- [ ] #3 Unsupported SOCKS4/4a semantics are not inferred from the Socks5 model
- [ ] #4 Malformed partial credentials fail before launch
- [ ] #5 Representative fixtures pass sing-box v1.13.21 check
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
