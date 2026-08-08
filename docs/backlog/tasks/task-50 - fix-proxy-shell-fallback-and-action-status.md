---
id: TASK-50
title: fix proxy shell fallback and action status
status: Done
assignee:
  - '@mahyar'
created_date: '2026-08-01 17:57'
updated_date: '2026-08-01 18:08'
labels:
  - cli
  - proxy-shell
  - bug
dependencies: []
priority: medium
ordinal: 8000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Fix two regressions in proxy shell commands. An explicitly requested http protocol currently fails when the connected runtime has only its default SOCKS inbound, even though the shell can use that endpoint. Also, enable/disable/toggle print status from the child process before the emitted script is applied to the caller's shell, so the first command reports the old environment and the next command reports the new one.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 proxy shell enable http succeeds with a connected runtime that has only a SOCKS inbound and emits a usable proxy URL
- [x] #2 an explicitly requested protocol uses its matching active inbound when available and falls back safely when it is unavailable
- [x] #3 enable, disable, and toggle status output describes the environment after the emitted script is applied
- [x] #4 regression tests cover protocol fallback and post-action status for bash/zsh and fish output helpers
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Adjust proxy endpoint selection to preserve usable fallback behavior for explicit protocols and update its tests/docs. 2. Make action status derive from the emitted script's resulting state and cover enable/disable/toggle branches with focused tests. 3. Run formatting, focused tests, clippy, and finalize the backlog task.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented explicit protocol fallback: matching HTTP/SOCKS inbounds remain preferred, but a single active inbound is reused with its real URL scheme. Action status now renders the post-script state; toggle-off derives the restored proxy value from the saved XRAT variables. Updated docs under docs/src/02-cli/proxy.md. Verification: cargo fmt --check; clippy --locked --workspace --all-targets -- -D warnings; proxy-shell tests 8 passed; proxy tests 54 passed; full suite with inherited proxy variables unset 665 passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Fixed proxy shell protocol fallback and stale action status. xrat proxy shell enable http now works with a SOCKS-only runtime using the usable socks5 endpoint; matching inbounds still win when available. enable, disable, and toggle now report the resulting shell state immediately, including restored toggle values. Updated documentation and regression tests. Verified with clean formatting, clippy, 665 full tests, and focused proxy tests.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
