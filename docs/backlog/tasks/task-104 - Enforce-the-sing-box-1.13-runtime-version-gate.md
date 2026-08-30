---
id: TASK-104
title: Enforce the sing-box 1.13 runtime version gate
status: To Do
assignee:
  - '@mhyrzt'
created_date: '2026-08-30 17:50'
labels:
  - sing-box
  - compatibility
  - runtime
milestone: m-7
dependencies:
  - TASK-98
references:
  - TASK-98
  - 'https://github.com/SagerNet/sing-box/releases/tag/v1.13.21'
priority: high
ordinal: 67000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement the approved version policy at every managed sing-box entry point. Parse native version output, accept stable 1.13.x, identify v1.13.21 as the conformance target, and reject older, newer-major/minor, or prerelease binaries before writing or launching a runtime session.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Stable sing-box versions >=1.13.0 and <1.14.0 are accepted
- [ ] #2 Pre-1.13, 1.14 prerelease/stable, malformed, and unqueryable versions are rejected before launch
- [ ] #3 Errors include detected version, supported range, configured binary path, and remediation
- [ ] #4 Connect, replace, probe, test, and scan paths apply the same policy
- [ ] #5 Parser and lifecycle tests cover accepted and rejected version strings
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
