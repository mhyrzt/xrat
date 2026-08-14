---
id: TASK-59
title: Release v0.14.0
status: In Progress
assignee:
  - '@codex'
created_date: '2026-08-14 20:42'
updated_date: '2026-08-14 20:45'
labels:
  - release
dependencies: []
priority: high
ordinal: 19000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Prepare and publish xrat v0.14.0 with restored TUI config and HTTPS subscription import, optional CLI subscription naming, and the config.toml settings editor.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Feature and documentation changes are committed in focused conventional commits
- [x] #2 Cargo.toml and Cargo.lock package versions are 0.14.0
- [x] #3 Release notes summarize user-visible behavior and compatibility
- [x] #4 just fmt ci passes before tagging
- [ ] #5 Annotated v0.14.0 tag is pushed and release automation completes successfully
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Review changes since v0.13.0 and write v0.14.0 release notes. 2. Bump Cargo package metadata to 0.14.0. 3. Run just fmt ci and commit release metadata. 4. Push master, create and push annotated v0.14.0, then monitor release automation.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Prepared v0.14.0 release notes and bumped Cargo.toml/Cargo.lock. Release gate passed on 2026-08-15: cargo fmt, Prettier, SQLFluff, cargo fmt --check, strict Clippy, and cargo test --locked with 727 tests passed.
<!-- SECTION:NOTES:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
