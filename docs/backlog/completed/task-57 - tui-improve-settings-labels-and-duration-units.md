---
id: TASK-57
title: 'tui: improve settings labels and duration units'
status: Done
assignee:
  - '@codex'
created_date: '2026-08-14 19:16'
updated_date: '2026-08-14 19:22'
labels: []
dependencies: []
modified_files:
  - src/app/config/editor.rs
  - src/tui/view/modals.rs
ordinal: 17000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make settings values easier to scan by showing duration units beside numeric values and replacing raw configuration-key labels with concise context-aware TUI labels.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Duration settings display the unit implied by their configuration field while their stored and edited values remain numeric
- [x] #2 Value labels are concise, title-cased, and omit redundant section context; parser.parse_mode is displayed as Mode
- [x] #3 Non-duration numeric settings do not receive a time unit
- [x] #4 Regression tests cover duration formatting and representative improved labels
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inspect the settings schema and value rendering to identify duration fields and label derivation. 2. Add presentation metadata/helpers for context-aware labels and duration suffixes without changing stored values. 3. Add focused regression tests and run formatting plus relevant test suites.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added presentation-only label humanization and exact-path duration units. Numeric edit/storage semantics are unchanged. No documentation update was needed because this only refines modal display. Validation: just fmt ci; 717 tests passed and clippy passed with warnings denied.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Settings now show concise sentence-case labels with preserved acronyms and context-specific names such as Parser > Mode. Duration values show their configured units, including fragment range endpoints, while unrelated numbers remain unitless. Verified by regression tests and the full CI suite.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
