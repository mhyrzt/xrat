---
id: TASK-81
title: Release v0.18.3
status: Done
assignee:
  - '@codex'
created_date: '2026-08-25 20:11'
updated_date: '2026-08-25 20:16'
labels:
  - release
dependencies: []
modified_files:
  - Cargo.toml
  - Cargo.lock
  - .github/RELEASE_NOTE.md
priority: high
ordinal: 43000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Publish a patch release containing the XHTTP share-link compatibility fix from TASK-80 so imported links with the inert legacy headerType=none parameter generate valid runtime configs.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Cargo package version and release notes identify v0.18.3
- [x] #2 Release notes accurately describe the XHTTP headerType compatibility fix and its narrow validation behavior
- [x] #3 just fmt ci passes on the versioned release tree
- [x] #4 Release metadata is committed and an annotated v0.18.3 tag is pushed with master
- [x] #5 The tag-triggered GitHub Release workflow is confirmed running on the tagged commit
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Review commits since v0.18.2 and prepare v0.18.3 package metadata and release notes. 2. Run just fmt ci and inspect the release diff. 3. Commit release metadata and create annotated tag v0.18.3. 4. Push master and the tag, confirm the Release workflow starts on the tagged commit, then finalize the release task.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Prepared v0.18.3 package metadata and release notes for neutral legacy XHTTP headerType compatibility. The versioned release tree passed just fmt ci: rustfmt, Prettier, SQLite/PostgreSQL SQL formatting, strict Clippy, 795 library tests, and 1 binary test. Committed release metadata as e4358bc and created annotated tag v0.18.3. Pushed master and the tag; remote master resolves to e4358bceece0ea2c56806093423ee46bcfb2e4c8, and GitHub Release run 32894344924 is in progress on that tagged commit.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Published the v0.18.3 release commit and annotated tag for XHTTP share-link compatibility. The full local gate passed, remote refs were verified, and the tag-triggered GitHub Release workflow is running on the expected commit.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
