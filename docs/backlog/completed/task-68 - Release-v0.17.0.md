---
id: TASK-68
title: Release v0.17.0
status: Done
assignee:
  - '@codex'
created_date: '2026-08-15 18:31'
updated_date: '2026-08-15 18:37'
labels:
  - release
dependencies: []
priority: high
ordinal: 28000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Publish v0.17.0 with managed proxy-core installation/update support and the Xray 26.3.27 preflight compatibility fix.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Cargo package version and release notes identify v0.17.0
- [x] #2 just fmt ci passes on the release commit
- [x] #3 An annotated v0.17.0 tag points to the release commit
- [x] #4 master and v0.17.0 are pushed to origin
- [x] #5 Tag-triggered GitHub release pipelines are confirmed queued or running
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Update package metadata and release notes for v0.17.0. 2. Run the full release gate and commit release metadata. 3. Create an annotated tag, push master and the tag, then confirm workflows were triggered without monitoring completion.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Prepared v0.17.0 package metadata and release notes covering managed core installation/update behavior and the Xray 26.3.27 preflight fix. Release gate passed: rustfmt, Prettier, SQLite/PostgreSQL SQL formatting, strict Clippy, 767 library tests, and 1 binary test.

Created annotated tag v0.17.0 at release commit e67445c09822b4cff2eabbf8f116701920ea1407 and pushed master plus the tag to origin. One-time GitHub Actions verification showed Release run 31901581360 in progress, CI run 31901579372 in progress, and Docs run 31901579386 completed successfully. Per user direction, workflows were not monitored further.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Published the v0.17.0 release commit and annotated tag with updated package metadata and release notes. The full local release gate passed, origin received master and the tag, and the expected Release, CI, and Docs workflows were confirmed triggered.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
