---
id: TASK-114
title: Release v0.19.1
status: In Progress
assignee:
  - '@codex'
created_date: '2026-09-11 22:15'
updated_date: '2026-09-11 22:20'
labels: []
dependencies: []
ordinal: 95000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Publish a patch release containing completed-session log retention and deleted-subscription config count fixes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Package version and release notes describe v0.19.1
- [x] #2 Required local validation passes
- [ ] #3 Annotated tag is pushed and release publication is verified
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Verify remote state and prepare version and notes. 2. Run just fmt ci. 3. Commit release changes and push master and annotated v0.19.1 tag. 4. Monitor release workflow and verify published artifacts.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Prepared v0.19.1 package and lockfile versions plus user-facing release notes. just fmt ci passed (814 tests). First run had a transient local HTTP request failure in executes_concurrent_downloads; isolated rerun and full gate both passed.
<!-- SECTION:NOTES:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
