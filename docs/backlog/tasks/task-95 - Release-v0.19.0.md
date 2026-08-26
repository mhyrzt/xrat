---
id: TASK-95
title: Release v0.19.0
status: Done
assignee:
  - '@codex'
created_date: '2026-08-26 10:08'
updated_date: '2026-08-26 10:14'
labels:
  - release
dependencies:
  - TASK-83
  - TASK-84
  - TASK-85
  - TASK-86
  - TASK-87
  - TASK-88
  - TASK-89
  - TASK-90
  - TASK-91
  - TASK-92
  - TASK-93
  - TASK-94
modified_files:
  - Cargo.toml
  - Cargo.lock
  - .github/RELEASE_NOTE.md
priority: high
ordinal: 57000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Publish a minor release containing Milestone 6 Xray schema compatibility, compatibility-targeted generation, and current VMess and VLESS share-link support.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Cargo package version and release notes identify v0.19.0
- [x] #2 Milestone 6 implementation is committed in focused conventional commits
- [x] #3 just fmt ci passes on the release commit
- [x] #4 An annotated v0.19.0 tag points to the release commit and master plus the tag are pushed
- [x] #5 The tag-triggered Release workflow is confirmed running on the tagged commit
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Split and commit schema, share-link, documentation, and Backlog changes. 2. Bump Cargo metadata to v0.19.0 and write release notes. 3. Run just fmt ci and commit release metadata. 4. Create annotated v0.19.0 tag and push master plus tag. 5. Confirm the tag-triggered Release workflow is running.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Committed the milestone as 1d90faf schema alignment, fd0b0da versioned config support, f1bd5b5 user documentation, aa98e33 prior Backlog archival, and 4460856 Milestone 6 completion. Prepared v0.19.0 in 9899fd7. The release tree passed just fmt ci with strict Clippy, 810 library tests, and 1 binary test. Created annotated tag v0.19.0 at 9899fd7 and pushed master plus the tag. GitHub Release workflow run 32957204478 started on the tagged commit and was in progress at handoff.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Published the v0.19.0 release commit and annotated tag for Xray schema compatibility and VMess AEAD support. Local gates passed, master and the tag were pushed, and GitHub Release workflow run 32957204478 was confirmed active.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
