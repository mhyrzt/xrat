---
id: TASK-79
title: Release v0.18.2
status: Done
assignee:
  - '@codex'
created_date: '2026-08-25 12:54'
updated_date: '2026-08-25 12:59'
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
ordinal: 41000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Publish a patch release containing the Xray transport-selector compatibility fix from TASK-77 and the explicit managed proxy-core installer from TASK-78.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Cargo package version and release notes identify v0.18.2
- [x] #2 Release notes accurately describe transport compatibility and explicit core installation
- [x] #3 just fmt ci passes on the versioned release tree
- [x] #4 Release metadata is committed and an annotated v0.18.2 tag is pushed with master
- [x] #5 The tag-triggered GitHub Release workflow is confirmed running on the tagged commit
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Review the issue feedback and all commits since v0.18.1. 2. Bump Cargo metadata to 0.18.2 and write release notes for TASK-77 and TASK-78. 3. Run just fmt ci and inspect the release diff. 4. Commit release metadata, create annotated tag v0.18.2, and push master plus tag. 5. Confirm the tag-triggered Release workflow starts on the tagged commit, then record and finalize the release task.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Prepared v0.18.2 package metadata and release notes covering dual network/method transport compatibility plus the explicit managed core installer. The versioned release tree passed just fmt ci: rustfmt, Prettier, SQLite/PostgreSQL SQL formatting, strict Clippy, 794 library tests, and 1 binary test.

Committed release metadata as bfca2a3 and created annotated tag v0.18.2 at bfca2a32270684dca5e7671bbd323ea940cb9aa3. Pushed master and the tag, then confirmed GitHub Release run 32850659280 is in progress on the tagged commit.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Published the v0.18.2 release commit and annotated tag for Xray transport compatibility and explicit managed core installation. The full local gate passed, remote refs were verified, and the tag-triggered GitHub Release workflow is running on the expected commit.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
