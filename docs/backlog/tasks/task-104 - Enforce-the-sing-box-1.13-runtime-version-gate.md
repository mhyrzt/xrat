---
id: TASK-104
title: Enforce the sing-box 1.13 runtime version gate
status: Done
assignee:
  - '@codex'
created_date: '2026-08-30 17:50'
updated_date: '2026-09-19 23:21'
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
- [x] #1 Stable sing-box versions >=1.13.0 and <1.14.0 are accepted
- [x] #2 Pre-1.13, 1.14 prerelease/stable, malformed, and unqueryable versions are rejected before launch
- [x] #3 Errors include detected version, supported range, configured binary path, and remediation
- [x] #4 Connect, replace, probe, test, and scan paths apply the same policy
- [x] #5 Parser and lifecycle tests cover accepted and rejected version strings
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Read the v1.13.21 tagged CLI source to model native version output and keep the TASK-98 stable-range policy. 2. Add one strict version parser and validator at the managed sing-box launch boundary before configuration is written. 3. Cover stable, prerelease, malformed, unavailable, and out-of-range versions with focused tests. 4. Trace test, probe, and scan callers; enforce the gate where they launch sing-box and record any path that does not yet use a sing-box runtime.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented the shared version --name gate from the tagged v1.13.21 CLI source. Managed connect and replacement preflight validate before creating config files; the test settings path validates before probe setup. Scan is TCP-only and has no sing-box process, while parse output only generates JSON and has no runtime launch. Validation passed: singbox::version tests, unsupported test-binary settings test, end-to-end managed sing-box lifecycle test, cargo clippy --all-targets -- -D warnings, and git diff --check. The lifecycle test requires a localhost socket bind and passed outside the restricted sandbox.

Policy update (2026-09-20): the gate now rejects only pre-1.13, malformed, or unavailable binaries. Any version >=1.13.0 is accepted; versions outside the tested >=1.13.0,<1.15.0 range (including prereleases) log a warning. Preflight remains mandatory and runs sing-box check with the actual binary. Implemented in src/singbox/version.rs with MINIMUM_VERSION and TESTED_RANGE.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added strict stable-1.13 sing-box version enforcement using native version --name output, with actionable errors and parser/lifecycle coverage. Connect, replace, and test paths now reject unsupported binaries before config generation or launch; scan and JSON-only parse have no sing-box runtime boundary.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
