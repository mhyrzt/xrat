---
id: TASK-102
title: Build a native sing-box conformance fixture matrix
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 15:36'
updated_date: '2026-08-30 17:52'
labels:
  - sing-box
  - testing
  - config-generation
  - ci
milestone: m-7
dependencies:
  - TASK-104
  - TASK-105
  - TASK-106
  - TASK-99
  - TASK-100
  - TASK-101
  - TASK-103
references:
  - TASK-73
  - 'https://github.com/SagerNet/sing-box'
  - 'https://github.com/yarikov/kvn-tui'
priority: high
ordinal: 64000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Expand native validation from a small optional DNS smoke test into a deterministic conformance matrix for every supported generated configuration. Cover probe and managed runtime shapes, all inbound combinations, outbounds, TLS/obfuscation, DNS, routing, and Clash API. Reuse the actual runtime preflight command and clearly distinguish skipped binary-unavailable checks from passing validation.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Fixtures cover every supported generated section and meaningful cross-section combination
- [ ] #2 Each fixture is checked with the supported sing-box binary using the same check command as runtime preflight
- [ ] #3 A missing validator is reported as skipped rather than counted as a passing conformance check
- [ ] #4 CI or release verification runs the matrix against the pinned supported sing-box version
- [ ] #5 A schema or native-validator rejection identifies the exact fixture and emitted JSON
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
