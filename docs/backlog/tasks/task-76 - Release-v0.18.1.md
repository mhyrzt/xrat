---
id: TASK-76
title: Release v0.18.1
status: Done
assignee:
  - '@codex'
created_date: '2026-08-23 11:25'
updated_date: '2026-08-23 11:32'
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
- [x] #2 The XHTTP fix and release metadata are committed on master
- [x] #3 just fmt ci passes on the release commit
- [x] #4 An annotated v0.18.1 tag points to the release commit and master plus the tag are pushed
- [x] #5 The tag-triggered Release workflow is confirmed running on the release commit
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Commit the completed TASK-75 implementation. 2. Bump Cargo metadata to v0.18.1 and replace RELEASE_NOTE.md with patch-release notes. 3. Run just fmt ci and commit the release metadata. 4. Create annotated tag v0.18.1 and push master plus the tag. 5. Confirm the tag-triggered Release workflow is running on the tagged commit, then hand publication over to automation without monitoring completion per user direction.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Prepared v0.18.1 package metadata and release notes for the XHTTP compatibility fix. Full release gate passed on the versioned tree: rustfmt, Prettier, SQLite/PostgreSQL SQL formatting, strict Clippy, 787 library tests, and 1 binary test.

Committed fix as 625566e and release metadata as c30ca27. Created annotated v0.18.1 tag at c30ca276603216f51f8e6755be12c610597124e5, pushed master and tag, and confirmed Release run 32636693520 is in progress.

Per user direction, stopped monitoring after confirming the tag-triggered pipeline was running. Before handoff, GitHub Actions CI and version-check jobs had passed; platform builds and Docker publication had started. Remaining publication is delegated to release automation.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Published the v0.18.1 release commit and annotated tag for the XHTTP compatibility fix. Local release gates passed, master and tag were pushed, and GitHub Release run 32636693520 was confirmed active with CI and version validation successful; pipeline completion monitoring was explicitly waived.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
