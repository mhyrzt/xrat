---
id: TASK-45
title: Add cargo-binstall metadata
status: To Do
assignee: []
created_date: '2026-07-07 13:50'
labels:
  - packaging
  - release
dependencies: []
priority: medium
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add Cargo.toml package metadata so released xrat binaries can be installed with cargo-binstall without building from source.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Cargo.toml includes cargo-binstall metadata for the published xrat release artifacts
- [ ] #2 The metadata matches the release archive naming and target triples produced by the release workflow
- [ ] #3 Documentation or release notes mention the cargo-binstall install path where appropriate
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
