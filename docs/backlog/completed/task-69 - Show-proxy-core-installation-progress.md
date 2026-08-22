---
id: TASK-69
title: Show proxy-core installation progress
status: Done
assignee:
  - '@codex'
created_date: '2026-08-15 18:43'
updated_date: '2026-08-15 19:16'
labels:
  - bug
  - setup
  - ux
dependencies: []
modified_files:
  - src/app/commands/progress.rs
  - src/app/commands/setup/cores.rs
  - src/app/commands/setup/mod.rs
  - docs/src/02-cli/setup.md
priority: high
ordinal: 29000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Proxy-core installation in xrat setup currently buffers the entire release archive silently after confirmation, leaving an apparently blank terminal. Stream downloads through the shared CLI progress UI and add useful install lifecycle logging while preserving checksum verification and rollback-safe activation.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Interactive setup displays byte progress while downloading each proxy core
- [x] #2 Non-TTY and machine-readable flows do not emit terminal progress artifacts
- [x] #3 Install logs identify core, version, download, verification, extraction, and activation outcomes
- [x] #4 Checksum verification and rollback-safe replacement behavior remain covered
- [x] #5 Focused tests and just fmt ci pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Reuse CliProgress for streamed core downloads with known and unknown lengths. 2. Add structured lifecycle logs around verification, extraction, and activation. 3. Add focused regression coverage and run the full project gate.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Core downloads now stream response chunks through CliProgress instead of buffering silently. Interactive table mode shows an immediate connection spinner followed by a byte bar when Content-Length is known or a byte-count spinner for chunked/unknown-length responses. JSON mode and non-TTY stderr disable progress. Structured tracing records install start/failure plus download, checksum, extraction, staged-version validation, and activation stages. Existing checksum and rollback tests remain green; new local HTTP regressions cover fixed-length and chunked downloads. Validation passed: 20 setup tests, focused progress/download tests, strict Clippy, just fmt ci, 769 library tests, and 1 binary test.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added visible streamed progress and structured lifecycle logging to proxy-core installation. Setup no longer appears frozen after confirmation, unknown-length responses remain informative, noninteractive output stays clean, and archive verification/rollback behavior is preserved.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
