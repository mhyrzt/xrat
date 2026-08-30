---
id: TASK-108
title: Publish the sing-box 1.13 support and limitation matrix
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:52'
labels:
  - sing-box
  - documentation
  - compatibility
milestone: m-7
dependencies:
  - TASK-99
  - TASK-100
  - TASK-101
  - TASK-103
  - TASK-105
  - TASK-106
references:
  - TASK-73
documentation:
  - docs/src/06-architecture/config-generation.md
priority: medium
ordinal: 82000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Publish user-facing and architecture documentation for the completed sing-box 1.13 backend. Separate import support, probe support, managed runtime support, inbound support, DNS/routing capabilities, required binary version, and intentionally rejected settings.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A protocol matrix distinguishes import, parse/show, probe, and managed-runtime support
- [ ] #2 Inbound, TLS/transport, DNS, routing, Clash API, and rule-set limitations are documented
- [ ] #3 The required stable 1.13.x range and v1.13.21 managed pin are visible in setup and troubleshooting docs
- [ ] #4 Every unsupported mapping includes the error users should expect and a viable alternative where one exists
- [ ] #5 Architecture documentation links each generated section to its owning module and conformance coverage
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
