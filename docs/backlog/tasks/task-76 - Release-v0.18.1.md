---
id: TASK-76
title: Release v0.18.1
status: In Progress
assignee:
  - '@codex'
created_date: '2026-08-23 11:25'
updated_date: '2026-08-23 11:29'
labels:
  - release
dependencies: []
references:
  - 'https://github.com/mhyrzt/xrat/issues/2'
modified_files:
  - Cargo.toml
  - Cargo.lock
  - .github/RELEASE_NOTE.md
priority: high
ordinal: 38000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Publish a patch release containing the XHTTP parameter compatibility fix and fail-safe typed Xray link parameter handling from TASK-75.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Cargo package version and release notes identify v0.18.1
- [ ] #2 The XHTTP fix and release metadata are committed on master
- [x] #3 just fmt ci passes on the release commit
- [ ] #4 An annotated v0.18.1 tag points to the release commit and master plus the tag are pushed
- [ ] #5 The tag-triggered Release workflow completes successfully
- [ ] #6 The published GitHub release and artifacts are available
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Commit the completed TASK-75 implementation. 2. Bump Cargo metadata to v0.18.1 and replace RELEASE_NOTE.md with patch-release notes. 3. Run just fmt ci and commit the release metadata. 4. Create annotated tag v0.18.1, push master and tag, then monitor the tag-triggered Release workflow through successful completion and verify the published release assets.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Prepared v0.18.1 package metadata and release notes for the XHTTP compatibility fix. Full release gate passed on the versioned tree: rustfmt, Prettier, SQLite/PostgreSQL SQL formatting, strict Clippy, 787 library tests, and 1 binary test.
<!-- SECTION:NOTES:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
