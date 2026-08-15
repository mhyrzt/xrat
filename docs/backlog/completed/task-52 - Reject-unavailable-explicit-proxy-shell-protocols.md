---
id: TASK-52
title: Reject unavailable explicit proxy shell protocols
status: Done
assignee:
  - '@codex'
created_date: '2026-08-08 12:06'
updated_date: '2026-08-08 12:10'
labels:
  - cli
  - proxy-shell
  - bug
dependencies: []
priority: medium
ordinal: 10000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Supersede TASK-50 explicit-protocol fallback behavior. When xrat proxy shell enable with an explicit protocol requests an inbound that is not active for the connected runtime, return an actionable error instead of silently exporting another protocol. Preserve TASK-50 post-action status improvements and default no-protocol cross-fallback.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Explicit http requires an active HTTP inbound and never falls back to SOCKS
- [x] #2 Explicit socks5 and socks5h require an active SOCKS inbound and never fall back to HTTP
- [x] #3 Errors identify the unavailable protocol and explain how to enable it or use automatic selection
- [x] #4 Omitting the protocol retains the existing safe cross-fallback behavior
- [x] #5 Tests and proxy shell documentation describe the strict explicit-protocol behavior
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Replace explicit protocol fallback with matching-inbound validation and actionable errors. 2. Update focused regression tests and proxy shell docs while preserving unrelated TASK-50 status changes. 3. Run focused tests and the repository CI gate.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Replaced TASK-50 explicit-protocol cross-fallback with strict matching-inbound selection while preserving automatic no-protocol fallback and TASK-50 post-action status work. Errors name the missing HTTP or SOCKS inbound, identify the config setting, and suggest reconnecting or omitting the protocol. Verification: 23 proxy-shell tests passed; just fmt ci passed; 679 full tests passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Explicit proxy shell protocols now require their matching active inbound and return actionable guidance when unavailable. Automatic selection still cross-falls back when no protocol is supplied. Documentation and regression tests were updated, with the full CI gate passing.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
