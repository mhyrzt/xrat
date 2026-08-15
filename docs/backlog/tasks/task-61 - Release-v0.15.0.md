---
id: TASK-61
title: Release v0.15.0
status: Done
assignee:
  - '@codex'
created_date: '2026-08-15 09:57'
updated_date: '2026-08-15 10:02'
labels:
  - release
dependencies: []
priority: high
ordinal: 21000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Prepare and publish xrat v0.15.0 with round-trip-safe imported config parameters, migration 0022, hardened rotation health and rollback behavior, and updated TUI rotation settings.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Feature, documentation, and backlog changes are committed in focused conventional commits
- [x] #2 Cargo.toml and Cargo.lock package versions are 0.15.0
- [x] #3 Release notes summarize user-visible behavior, migration impact, and compatibility
- [x] #4 just ci passes before tagging
- [x] #5 Annotated v0.15.0 tag and master are pushed without waiting for release automation
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Review changes since v0.14.0 and prepare release notes. 2. Bump Cargo package metadata to 0.15.0. 3. Run just ci and commit release metadata. 4. Create and push the annotated v0.15.0 tag and master without monitoring CI.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Prepared v0.15.0 release notes and bumped Cargo.toml/Cargo.lock. Release gate passed on 2026-08-15: cargo fmt --check, strict Clippy, and cargo test --locked with 737 tests passed.

Release commit 162c3c1 and annotated v0.15.0 tag were pushed to origin. Per user direction, release automation was not monitored.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Published the v0.15.0 release trigger with migration-aware release notes and package metadata. Verified locally with just ci and 737 passing tests; pipeline and publication outcomes remain intentionally unmonitored.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
