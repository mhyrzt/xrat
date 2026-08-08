---
id: TASK-53
title: Release v0.13.0
status: In Progress
assignee:
  - '@codex'
created_date: '2026-08-08 12:27'
labels:
  - release
dependencies: []
priority: high
ordinal: 11000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Prepare and publish xrat v0.13.0 with configurable real-delay HTTP status acceptance, redirect handling, strict explicit proxy-shell protocols, and corrected post-action proxy-shell status reporting.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Feature and fix changes are committed in focused conventional commits
- [ ] #2 Cargo.toml and Cargo.lock package versions are 0.13.0
- [ ] #3 Release notes summarize user-visible behavior and upgrade compatibility
- [ ] #4 just fmt ci passes before tagging
- [ ] #5 Annotated v0.13.0 tag is pushed and release automation is monitored
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Split proxy-shell and real-delay changes into focused commits. 2. Bump package version and write v0.13.0 release notes. 3. Run just fmt ci and commit release metadata. 4. Push master, create and push annotated v0.13.0 tag, then monitor the release workflow.
<!-- SECTION:PLAN:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
