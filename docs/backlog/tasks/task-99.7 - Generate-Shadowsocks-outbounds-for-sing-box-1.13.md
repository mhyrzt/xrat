---
id: TASK-99.7
title: Generate Shadowsocks outbounds for sing-box 1.13
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:51'
labels:
  - sing-box
  - shadowsocks
  - outbound
milestone: m-7
dependencies:
  - TASK-98
references:
  - 'https://sing-box.sagernet.org/configuration/outbound/shadowsocks/'
parent_task_id: TASK-99
priority: high
ordinal: 75000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add Shadowsocks managed-runtime generation for the methods Xrat can import and sing-box 1.13 supports. Validate method/password requirements and explicitly handle SIP002 plugin options instead of silently dropping them.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Supported AEAD and AEAD-2022 methods map to exact sing-box method names
- [ ] #2 Password/key requirements are validated per method family
- [ ] #3 Unsupported legacy ciphers and plugin options fail before launch unless an exact supported mapping exists
- [ ] #4 SIP002 and normalized database records generate equivalent output
- [ ] #5 Representative fixtures pass sing-box v1.13.21 check
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
