---
id: TASK-99.1
title: Build shared sing-box 1.13 TLS and transport mapping
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:50'
labels:
  - sing-box
  - tls
  - transport
  - config-generation
milestone: m-7
dependencies:
  - TASK-98
references:
  - 'https://sing-box.sagernet.org/configuration/shared/tls/'
  - 'https://sing-box.sagernet.org/configuration/shared/v2ray-transport/'
  - 'https://github.com/yarikov/kvn-tui'
parent_task_id: TASK-99
priority: high
ordinal: 66000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Define reusable typed builders for the TLS and V2Ray transport fields shared by VLESS, VMess, and Trojan without importing Xray-specific wire assumptions. Map SNI, insecure, ALPN, uTLS, REALITY, WebSocket, gRPC, HTTP, and HTTPUpgrade only when normalized Xrat data is sufficient for exact sing-box 1.13 output.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each accepted TLS and transport input has a documented Xrat-to-sing-box field mapping
- [ ] #2 TLS-disabled profiles do not receive an enabled TLS block
- [ ] #3 REALITY requires all sing-box-required fields and never guesses missing values
- [ ] #4 WebSocket, gRPC, HTTP, and HTTPUpgrade preserve path, host, headers, and service names where supported
- [ ] #5 Xray-only or lossy transport settings fail before launch with field-specific diagnostics
- [ ] #6 Fixtures pass sing-box v1.13.21 check
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
