---
id: TASK-96
title: Exclude deleted configs from subscription counts
status: Done
assignee:
  - '@codex'
created_date: '2026-08-26 11:12'
updated_date: '2026-08-26 11:16'
labels: []
dependencies: []
modified_files:
  - src/db/repository/subscriptions.rs
  - src/db/database/tests/import_cases/reconcile.rs
ordinal: 58000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Subscription list and TUI counts must report active configs only. Provider refresh keeps removed configs as soft-deleted history, so counting every joined row inflates the displayed total.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Subscription config_count excludes soft-deleted configs
- [x] #2 Subscriptions with zero active configs remain listed with count zero
- [x] #3 Regression tests cover active and soft-deleted subscription configs
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add a database regression test covering active, soft-deleted, and zero-active subscription counts. 2. Filter the subscription list join to active configs only. 3. Run focused database tests and just fmt ci; record results and finalize the task.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Filtered subscription joins by configs.is_deleted = 0 across SQLite and Postgres lookup/list queries. Validation: focused reconciliation regression passed; just fmt ci passed with Clippy clean, 810 library tests and 1 binary test.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Subscription list and lookup counts now include active configs only while retaining subscriptions with zero active configs. Regression coverage verifies counts after provider reconciliation and after all configs are soft-deleted. No migration or user documentation change is required.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
