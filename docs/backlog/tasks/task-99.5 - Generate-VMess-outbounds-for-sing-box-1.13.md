---
id: TASK-99.5
title: Generate VMess outbounds for sing-box 1.13
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:51'
labels:
  - sing-box
  - vmess
  - outbound
milestone: m-7
dependencies:
  - TASK-99.1
references:
  - 'https://sing-box.sagernet.org/configuration/outbound/vmess/'
parent_task_id: TASK-99
priority: high
ordinal: 73000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add VMess managed-runtime generation from normalized base64-JSON and URI imports. Map UUID, security cipher, alter_id, packet encoding, TLS, and supported transports using sing-box 1.13 semantics.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 AEAD VMess profiles emit valid UUID, security, alter_id, and packet encoding fields
- [ ] #2 Supported TLS and transports use the shared typed mapping
- [ ] #3 Legacy or unsupported cipher and transport combinations fail with actionable errors
- [ ] #4 Both supported VMess import forms produce equivalent runtime output
- [ ] #5 Representative fixtures pass sing-box v1.13.21 check
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
