---
id: TASK-118
title: Release v0.20.0 with expanded sing-box support
status: Done
assignee:
  - '@codex'
created_date: '2026-09-21 20:00'
updated_date: '2026-09-21 23:37'
labels:
  - release
  - sing-box
dependencies: []
ordinal: 99000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Publish a minor release for managed sing-box generation and probes across imported protocols, Xray Hysteria2 outbound support, and the setup version pin. State the current conformance limits accurately.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Cargo package and lockfile versions are 0.20.0 and release notes accurately describe shipped behavior and limitations
- [x] #2 Required local validation passes and recorded results distinguish native sing-box checks from the pending fixture matrix
- [x] #3 An annotated v0.20.0 tag is pushed and GitHub release artifacts, Docker image, and crate publication are verified or explicitly reported
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inspect release scope and current remote state. 2. Prepare version, release notes, and directly affected docs. 3. Run just fmt ci and native sing-box checks. 4. Commit and push master, tag v0.20.0, monitor publication, and report evidence.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Prepared v0.20.0 package and lockfile versions, release notes, and corrected stale sing-box documentation. Fixed the fake Xray test fixture to recognize run -test preflight. just fmt ci passed: formatting, Clippy, 853 library tests, 1 binary test. Official sing-box v1.13.21 glibc archive SHA-256 matched its GitHub digest; all 9 current native validator tests passed with that binary. Full fixture matrix and pinned CI gate remain TASK-102/TASK-109.

Annotated v0.20.0 tag and master commits pushed. Release workflow 35650325726 completed successfully; GitHub release contains SHASUMS256.txt and four platform archives. GHCR tag 0.20.0 resolves to linux/amd64 and linux/arm64 manifests. cargo info xrat@0.20.0 from /tmp downloaded the published crate. README engine matrix followed as a separate master docs commit ec8dfb6 after the release tag.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Published xrat v0.20.0 with expanded sing-box support and native Xray Hysteria2. Local just fmt ci and nine native sing-box 1.13.21 checks passed; GitHub release assets, GHCR multi-arch image, and crates.io package were verified. Full sing-box conformance matrix and pinned CI gate remain separate backlog work.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
