---
id: TASK-74
title: Release v0.18.0
status: Done
assignee:
  - '@codex'
created_date: '2026-08-23 10:41'
updated_date: '2026-08-23 10:46'
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
- [x] #3 An annotated v0.18.0 tag points to the release commit
- [x] #4 master and v0.18.0 are pushed to origin
- [x] #5 Tag-triggered GitHub release pipelines are confirmed queued or running
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Update package metadata and release notes for v0.18.0. 2. Run the full release gate and commit release metadata. 3. Create an annotated tag, push master and the tag, then confirm workflows were triggered without monitoring completion.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Prepared v0.18.0 package metadata and release notes covering managed DNS behavior, Xray probe DNS settings, TUI editing, sing-box validation limits, and managed core download progress. Release gate passed: rustfmt, Prettier, SQLite/PostgreSQL SQL formatting, strict Clippy, 781 library tests, and 1 binary test.

Created annotated tag v0.18.0 at release commit 25c4d35a867218748e8ba466d8a1f89f59ef1504 and pushed master plus the tag to origin. GitHub Actions verification showed Release run 32634646477, CI run 32634646052, and Docs run 32634646061 in progress.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Published the v0.18.0 release commit and annotated tag with updated package metadata and release notes. The full local release gate passed, origin received master and the tag, and the expected Release, CI, and Docs workflows were confirmed triggered.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
