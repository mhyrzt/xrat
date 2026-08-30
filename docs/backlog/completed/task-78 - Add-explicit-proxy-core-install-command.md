---
id: TASK-78
title: Add explicit proxy-core install command
status: Done
assignee:
  - '@codex'
created_date: '2026-08-25 11:56'
updated_date: '2026-08-25 12:23'
labels: []
dependencies: []
ordinal: 40000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Expose the existing managed proxy-core installation capability as an explicit CLI command so users can install Xray, V2Ray, or sing-box directly from upstream GitHub releases and optionally pin a release version.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 xrat install supports selecting Xray, V2Ray, or sing-box
- [x] #2 Users can request a specific upstream release version and omit it to install latest
- [x] #3 Installed binaries use the existing managed binary location and update runtime configuration consistently
- [x] #4 CLI parsing and release-selection behavior are covered by focused tests
- [x] #5 User-facing CLI documentation includes install examples
- [x] #6 Users can explicitly install the newest published prerelease without specifying its version
- [x] #7 Install output and logs identify the selected core, version, and installed binary path
- [x] #8 Downloads display progress when terminal output is enabled
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add a prerelease selector that conflicts with exact version pinning. 2. Resolve the newest non-draft prerelease from the upstream GitHub releases API. 3. Verify and improve progress, logging, final output, and config path persistence. 4. Add tests and documentation. 5. Run focused checks and just fmt ci.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added xrat install CORE with optional version selection, exact GitHub tag resolution, parser and unit coverage, and CLI documentation. Focused CLI tests: 4 passed. Release URL test: 1 passed. Help output manually verified.

Final validation passed: just fmt ci; 792 library tests and 1 additional test passed, Clippy passed with warnings denied, formatting and documentation linting passed, and git diff check passed.

Added prerelease selection with an exact-version conflict, latest-prerelease resolution by GitHub creation time, clearer pre-download and completion output, and expanded persisted event details. Verified 5 install parser tests, 15 core installer tests, config path update regression test, help output, and git diff check.

Follow-up validation passed: just fmt ci; 794 library tests and 1 additional test passed, Clippy passed with warnings denied, formatting and documentation linting passed, and git diff check passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added explicit newest-prerelease installation, preserved stable-by-default and exact-version behavior, and verified interactive download progress. Installation now announces the selected source and reports the installed core, version, binary path, and updated config path; structured tracing and the persisted install event carry the same operational details. Managed binary paths remain atomically pinned in config.toml.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
