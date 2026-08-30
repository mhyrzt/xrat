---
id: TASK-99.3
title: Generate VLESS outbounds for sing-box 1.13
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:51'
labels:
  - sing-box
  - vless
  - outbound
milestone: m-7
dependencies:
  - TASK-99.1
references:
  - 'https://sing-box.sagernet.org/configuration/outbound/vless/'
parent_task_id: TASK-99
priority: high
ordinal: 71000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add VLESS managed-runtime generation from normalized Xrat nodes. Cover UUID, flow, packet encoding, TLS/REALITY, and supported transports without carrying Xray-only field names or defaults into sing-box output.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Plain, TLS, and REALITY VLESS profiles generate only documented sing-box 1.13 fields
- [ ] #2 XTLS Vision flow and packet encoding are preserved only when valid
- [ ] #3 Supported WebSocket, gRPC, HTTP, and HTTPUpgrade profiles use the shared transport mapping
- [ ] #4 Missing UUID, incomplete REALITY, and unsupported transport/security combinations fail before launch
- [ ] #5 Representative fixtures pass sing-box v1.13.21 check
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
