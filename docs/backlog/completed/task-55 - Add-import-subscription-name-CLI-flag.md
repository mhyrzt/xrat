---
id: TASK-55
title: Add import subscription name CLI flag
status: Done
assignee:
  - '@codex'
created_date: '2026-08-14 13:34'
updated_date: '2026-08-14 13:39'
labels:
  - cli
  - import
dependencies: []
priority: medium
ordinal: 13000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Allow xrat import callers to assign a subscription name with --name or -n while preserving existing behavior when omitted.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 xrat import accepts --name and -n with a non-empty value
- [x] #2 The supplied name is persisted for a new imported source
- [x] #3 Re-importing an existing subscription URL with --name updates its name without duplicating the subscription
- [x] #4 Omitting --name preserves existing import behavior
- [x] #5 CLI help and import documentation describe the option
- [x] #6 Tests cover long and short parsing, blank-name rejection, and persistence
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Extend ImportArgs and command dispatch with an optional name. 2. Apply the trimmed name to the import source and resulting subscription, including existing URLs. 3. Add parser/persistence tests and docs. 4. Run just fmt ci and finalize.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented optional --name/-n parsing with trimming and blank-value rejection. Shared import persistence now applies the supplied name to new subscriptions and renames an existing matching URL without duplication; omission preserves prior behavior. Validation: import-focused tests passed (44 tests), help output verified, and just fmt ci passed with strict Clippy and 694 tests.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added xrat import --name/-n for naming subscriptions, including safe rename behavior on re-import. Updated help/docs and parser/persistence tests; just fmt ci passed (694 tests).
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
