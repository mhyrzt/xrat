---
id: TASK-74
title: Release v0.18.0
status: In Progress
assignee:
  - '@codex'
created_date: '2026-08-23 10:41'
updated_date: '2026-08-23 10:45'
labels:
  - release
dependencies: []
priority: high
ordinal: 36000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Publish v0.18.0 with managed DNS settings across runtime engines and Xray probes, editable DNS settings in the TUI, and visible managed core download progress.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Cargo package version and release notes identify v0.18.0
- [x] #2 just fmt ci passes on the release commit
- [ ] #3 An annotated v0.18.0 tag points to the release commit
- [ ] #4 master and v0.18.0 are pushed to origin
- [ ] #5 Tag-triggered GitHub release pipelines are confirmed queued or running
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Update package metadata and release notes for v0.18.0. 2. Run the full release gate and commit release metadata. 3. Create an annotated tag, push master and the tag, then confirm workflows were triggered without monitoring completion.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Prepared v0.18.0 package metadata and release notes covering managed DNS behavior, Xray probe DNS settings, TUI editing, sing-box validation limits, and managed core download progress. Release gate passed: rustfmt, Prettier, SQLite/PostgreSQL SQL formatting, strict Clippy, 781 library tests, and 1 binary test.
<!-- SECTION:NOTES:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
