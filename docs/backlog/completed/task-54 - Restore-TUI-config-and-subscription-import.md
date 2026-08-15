---
id: TASK-54
title: Restore TUI config and subscription import
status: Done
assignee:
  - '@codex'
created_date: '2026-08-14 13:04'
updated_date: '2026-08-14 13:25'
labels:
  - tui
  - import
dependencies: []
priority: medium
ordinal: 12000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Restore a compact in-app import flow so users can press i from either TUI tab, paste one config share link or HTTP(S) subscription URL, optionally name subscriptions, and persist the result without leaving the TUI.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Global i opens an import modal from both Configs and Subscriptions tabs
- [x] #2 A supported single config link is validated, saved, and reflected in reloaded TUI data
- [x] #3 An HTTP(S) subscription URL prompts for a name and uses a generated placeholder when left blank
- [x] #4 Invalid or unsupported pasted input reports a clear error without exiting the TUI
- [x] #5 TUI help and user documentation describe the import flow
- [x] #6 Tests cover modal state, key handling, input classification, naming, and persistence
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add two-stage import modal state, global key routing, and bracketed paste handling. 2. Add background config/subscription persistence using existing import services and reload TUI data. 3. Update help/docs and add regression tests. 4. Run just fmt ci and finalize the task.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented global i import flow with bracketed paste handling, two-stage subscription naming, HTTPS/HTTP URL detection, generated fallback names, asynchronous persistence, data reload, and inline validation. Verification: focused TUI suite passed 116 tests; just fmt ci passed strict Clippy and all 690 tests.

User requested shortening the generated fallback name from subscription-<random> to sub-<random>.

Adjusted the generated blank-name fallback to sub-<6 random hex characters>. Verification: tui::run::tests passed 4/4 and git diff --check passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Restored TUI import for single config links and HTTP(S) subscription URLs. Subscriptions receive a typed or generated name, duplicate URLs are reused, successful imports reload TUI data, and failures remain non-fatal. Updated help/docs and verified with just fmt ci (690 tests passed).

Generated fallback subscription names now use the shorter sub-<random> form.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
