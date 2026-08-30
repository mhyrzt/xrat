---
id: TASK-106
title: Audit sing-box top-level log and Clash API output
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:50'
labels:
  - sing-box
  - clash-api
  - logging
  - config-generation
milestone: m-7
dependencies:
  - TASK-98
references:
  - 'https://sing-box.sagernet.org/configuration/experimental/clash-api/'
  - TASK-73
priority: medium
ordinal: 69000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Verify the generated top-level log and experimental.clash_api objects against sing-box 1.13, including timestamp behavior, controller binding, authentication secret handling, optional-section omission, and collisions between API and local proxy ports.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Generated log fields are valid for sing-box 1.13 and preserve TUI log parsing requirements
- [ ] #2 Clash API is omitted when statistics are disabled
- [ ] #3 Enabled Clash API binds the configured endpoint and preserves configured authentication policy
- [ ] #4 Port collisions and unsafe non-loopback exposure are rejected or explicitly authorized
- [ ] #5 Enabled and disabled fixtures pass sing-box v1.13.21 check
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
