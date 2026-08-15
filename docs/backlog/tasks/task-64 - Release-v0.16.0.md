---
id: TASK-64
title: Release v0.16.0
status: In Progress
assignee:
  - '@codex'
created_date: '2026-08-15 11:58'
updated_date: '2026-08-15 12:03'
labels:
  - release
dependencies: []
priority: high
ordinal: 24000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Prepare and publish xrat v0.16.0 with managed runtime routing for Xray/V2Ray and sing-box, safe manual replacement preflight, and updated settings documentation.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Feature, documentation, and backlog changes are committed in focused conventional commits
- [x] #2 Cargo.toml and Cargo.lock package versions are 0.16.0
- [x] #3 Release notes summarize routing behavior, engine limitations, and compatibility
- [x] #4 just ci passes before tagging
- [ ] #5 Annotated v0.16.0 tag and master are pushed without waiting for release automation
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Review commits since v0.15.0 and prepare routing-focused release notes. 2. Bump Cargo.toml and Cargo.lock to 0.16.0. 3. Run just ci and commit release metadata. 4. Create and push annotated v0.16.0 plus master without monitoring release automation.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Prepared v0.16.0 routing release notes and bumped Cargo.toml/Cargo.lock. The first just fmt ci run hit a transient /proc argument-read race in resolves_exe_and_cmd_for_spawned_process; the focused test passed immediately, followed by a clean just ci run with strict Clippy and all 747 tests passing.
<!-- SECTION:NOTES:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
