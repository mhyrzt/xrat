---
id: TASK-119
title: Reuse release binaries in multi-platform Docker images
status: Done
assignee:
  - '@codex'
created_date: '2026-09-21 23:30'
updated_date: '2026-09-22 00:25'
labels:
  - docker
  - ci
  - release
  - performance
dependencies: []
ordinal: 100000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Avoid recompiling Rust under Docker/QEMU for each release image architecture. Keep local docker build from source working, reduce build context, and preserve published runtime behavior.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Local Dockerfile default target still builds xrat from source and includes Xray and sing-box runtime assets
- [x] #2 Docker build context excludes unrelated repository content, and workflow/Dockerfile validation passes
- [x] #3 Docs explain local and release build paths, and the task handoff records that next-tag CI publish remains unverified
- [x] #4 Release Docker workflow uses checked Linux musl artifacts from the release build matrix and is configured to publish amd64 and arm64 images without recompiling xrat
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Split the Dockerfile into source-build and artifact-based image targets with shared runtime setup. 2. Download and verify Linux archives in release CI, then build the artifact-based multi-platform target. 3. Add a narrow .dockerignore and update docs. 4. Validate syntax, artifact mapping, and a local build where available.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Release job now downloads tagged Linux musl archives from the existing build matrix, checks both extracted binaries and architectures, and builds the release target for amd64 and arm64. Default local target still compiles source; Cargo registry and target directories use BuildKit caches. A whitelist .dockerignore limits context. Validated v0.20.0 archive checksums locally, built and ran the release image on amd64, built and ran the local source image on amd64, confirmed Xray and sing-box in the local image, passed Buildx checks for both release platforms and the local target, actionlint, git diff --check, and just fmt ci (853 library tests plus 1 binary test). Arm64 runtime image and hosted tagged workflow were not run locally; verify on the next release.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Reused release-built Linux binaries for multi-platform images and cached local source builds. Both amd64 build paths passed local smoke tests; the next tagged CI run will validate hosted arm64 publication and time savings.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
