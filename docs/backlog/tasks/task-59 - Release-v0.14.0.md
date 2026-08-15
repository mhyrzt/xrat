---
id: TASK-59
title: Release v0.14.0
status: Done
assignee:
  - '@codex'
created_date: '2026-08-14 20:42'
updated_date: '2026-08-14 22:00'
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
- [x] #5 Annotated v0.14.0 tag is pushed; GitHub release and crates.io publication complete successfully, while GHCR may finish asynchronously
<!-- AC:END -->



## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Review changes since v0.13.0 and write v0.14.0 release notes. 2. Bump Cargo package metadata to 0.14.0. 3. Run just fmt ci and commit release metadata. 4. Push master, create and push annotated v0.14.0, then monitor release automation.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Prepared v0.14.0 release notes and bumped Cargo.toml/Cargo.lock. Release gate passed on 2026-08-15: cargo fmt, Prettier, SQLFluff, cargo fmt --check, strict Clippy, and cargo test --locked with 727 tests passed.

The annotated v0.14.0 tag, GitHub release, platform archives, checksums, and crates.io publication succeeded. The user accepted closing the release task while the unusually long GHCR multi-architecture build remains in progress; monitor it separately if needed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Published xrat v0.14.0 with the restored TUI import flow, optional subscription naming, and settings editor. Verified locally with just fmt ci and 727 passing tests; GitHub release and crates.io publication succeeded. Residual risk: the GHCR multi-architecture image build was still running when this task was closed by user decision.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
