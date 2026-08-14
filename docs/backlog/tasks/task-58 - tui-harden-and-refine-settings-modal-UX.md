---
id: TASK-58
title: 'tui: harden and refine settings modal UX'
status: Done
assignee:
  - '@codex'
created_date: '2026-08-14 19:37'
updated_date: '2026-08-14 20:14'
labels: []
dependencies: []
modified_files:
  - src/app/config/editor.rs
  - src/app/config/mod.rs
  - src/tui/app/settings.rs
  - src/tui/app/types.rs
  - src/tui/app/tests/settings.rs
  - src/tui/keymap/mod.rs
  - src/tui/keymap/tests/modals.rs
  - src/tui/run/mod.rs
  - src/tui/view/modals.rs
  - docs/src/02-cli/tui.md
ordinal: 18000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Address the settings-modal review findings so editing is safe, focus behavior is predictable, unsupported settings are honest, and the modal remains usable on compact terminals.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Modified character shortcuts never insert accidental text; Ctrl+S behavior is safe and mode-aware
- [x] #2 Reset only affects a setting while the Values pane is focused
- [x] #3 DNS settings are visibly unavailable and cannot be edited until runtime generation supports them
- [x] #4 Section and value selections remain visible while navigating constrained terminal heights
- [x] #5 Compact terminals use readable panes, help, and key hints without clipping important content
- [x] #6 Saving keeps the settings modal open, reports the result, and avoids rewriting when nothing changed
- [x] #7 The modal distinguishes inherited defaults, explicit overrides, and unsaved changes and shows default values in help
- [x] #8 Regression tests cover keyboard safety, focus-scoped reset, unavailable settings, saving, scrolling, and compact rendering
- [x] #9 Value-state markers use subtle visual weight and a visible Help-pane legend explains each marker
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Replace the heavy explicit-override glyph with a subtle marker while preserving distinct default and unsaved states. 2. Add a concise marker legend to the Help pane and update documentation. 3. Add rendering assertions and run focused settings validation.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented mode-safe shortcuts, Values-only reset, read-only inactive DNS presentation, auto-following section navigation, compact single-pane rendering, persistent save feedback, no-op save protection, and default/override/dirty metadata. Validation: cargo fmt and clippy --locked --all-targets -D warnings passed; 30 focused settings tests and the unchanged-save inode regression passed. The full suite ran 725 tests successfully; one process-inspection test was flaky but passed in isolation, while connect_and_disconnect_persist_direct_transition_metadata remains environmentally blocked because the user's active Xray process owns port 18200.

Follow-up UX refinement: replaced the visually heavy explicit-override circle with '+', added the full marker legend to the Help pane, and updated TUI documentation. Validation: cargo fmt --check, 30 focused settings tests, dedicated legend rendering test, and clippy with warnings denied all passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Hardened and refined the settings modal: safe shortcuts and reset behavior, persistent save feedback, responsive panes and scrolling, honest inactive DNS controls, default/source metadata, and a lightweight documented marker system (middle dot inherited, plus override, asterisk unsaved).
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
