---
id: TASK-45
title: Add cargo-binstall metadata
status: Done
assignee: []
created_date: '2026-07-07 13:50'
updated_date: '2026-07-11 23:51'
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
- [x] #1 Cargo.toml includes cargo-binstall metadata for the published xrat release artifacts
- [x] #2 The metadata matches the release archive naming and target triples produced by the release workflow
- [x] #3 Documentation or release notes mention the cargo-binstall install path where appropriate
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added [package.metadata.binstall] to Cargo.toml with pkg-url matching the release workflow's archive naming (xrat-v{version}-{target}.tar.gz, tgz format, binary at archive root). Verified with cargo binstall --manifest-path . --dry-run against the real published v0.10.0 GitHub release — resolved and would install correctly. Documented cargo binstall xrat as the recommended Cargo install path in docs/src/01-getting-started/cargo-install.md and README.md, ahead of the source-building cargo install path. No workflow or code changes needed; existing target triples (x86_64/aarch64 linux-musl, x86_64/aarch64 apple-darwin) and SHASUMS256.txt already matched binstall's requirements. Residual risk: none identified; only maintenance concern is keeping the pkg-url template in sync if the release workflow's archive naming ever changes.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
