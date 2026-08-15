---
id: TASK-67
title: Fix Xray 26 runtime validation regression
status: Done
assignee:
  - '@codex'
created_date: '2026-08-15 18:20'
updated_date: '2026-08-15 18:25'
labels:
  - bug
  - runtime
dependencies: []
modified_files:
  - src/app/runtime_service/spawn.rs
  - src/app/runtime_service/tests/connect_status_cases/unit_cases.rs
priority: high
ordinal: 27000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
XRAT v0.16.1 tests imported configs successfully but every managed runtime start fails Xray 26.3.27 native config validation with exit status 23. Determine which recent runtime/core-management change caused the shared generated config or invocation to become invalid, preserve actionable validator diagnostics, and restore connections.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A generated managed Xray runtime config validates with Xray 26.3.27
- [x] #2 Runtime start/connect no longer fails for all otherwise valid configs
- [x] #3 Regression coverage exercises the failing validation or invocation contract
- [x] #4 Relevant focused tests and the project verification gate pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Reproduce the exact Xray validator failure and compare recent runtime/core commits. 2. Add a focused regression test and apply the smallest fix. 3. Run focused validation and just fmt ci.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Root cause: commit 25bf764 introduced native preflight with tempfile::NamedTempFile::new_in, producing extensionless paths. Xray 26.3.27 selects the config parser by filename and exits 23 for those paths, while the identical retained session-N.json validates successfully. Changed preflight tempfiles to retain automatic cleanup while using a .json suffix. Added a regression validator that reproduces exit 23 for extensionless paths. Validation: focused regression passed; 32 runtime-service tests passed; just fmt ci passed with strict Clippy, 767 library tests, and 1 binary test; git diff --check is clean. No user documentation change is needed because this restores intended runtime behavior.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Fixed the Xray 26.3.27 connection regression by giving native preflight tempfiles a .json suffix so Xray can select its JSON parser. Added direct regression coverage for the exit-23 failure and passed the complete project gate.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
