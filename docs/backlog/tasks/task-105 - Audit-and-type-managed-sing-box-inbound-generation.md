---
id: TASK-105
title: Audit and type managed sing-box inbound generation
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:50'
labels:
  - sing-box
  - inbound
  - config-generation
milestone: m-7
dependencies:
  - TASK-98
references:
  - 'https://sing-box.sagernet.org/configuration/inbound/'
  - TASK-73
priority: high
ordinal: 68000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the overly broad shared inbound shape with schema-correct typed generation for SOCKS, HTTP, and Shadowsocks managed inbounds. Audit listen fields, network restrictions, authentication users, cipher/method, and readiness endpoints against sing-box 1.13.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 SOCKS, HTTP, and Shadowsocks inbounds emit only fields valid for their own schemas
- [ ] #2 SOCKS username/password authentication is represented using the documented users shape
- [ ] #3 HTTP and Shadowsocks authentication behavior is either implemented exactly or rejected explicitly
- [ ] #4 TCP/UDP network choices preserve configured behavior and invalid combinations fail preflight
- [ ] #5 Each individual and combined inbound configuration passes sing-box v1.13.21 check
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
