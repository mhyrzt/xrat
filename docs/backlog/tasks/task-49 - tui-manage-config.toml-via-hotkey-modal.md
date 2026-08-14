---
id: TASK-49
title: 'tui: manage config.toml via hotkey + modal'
status: Done
assignee:
  - '@codex'
created_date: '2026-07-31 21:08'
updated_date: '2026-08-14 17:30'
labels:
  - tui
  - config
dependencies: []
ordinal: 7000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a new TUI hotkey and modal for managing and modifying config.toml (settings/config file) directly from the TUI.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 New hotkey opens a config.toml management modal in the TUI
- [x] #2 Modal supports viewing and editing config.toml values
- [x] #3 Changes persist to config.toml and are reflected without full restart where feasible
- [x] #4 Hotkey and keymap documented in TUI help
- [x] #5 Tests cover modal state transitions and save path
- [x] #6 Section pane renders capitalized hierarchical labels with tree indentation
- [x] #7 Values pane supports independent Up/Down navigation when focused
- [x] #8 Third-level setting groups are folded into their parent Values pane with readable subheaders
- [x] #9 Boolean values render as glyphs instead of true/false text in the settings modal
- [x] #10 Empty list values render as a human-readable empty state instead of raw brackets
- [x] #11 Zero-valued automatic test concurrency settings render as auto without changing unrelated numeric zeros
- [x] #12 Fragment range lists render as indented min/max rows while remaining single editable settings
- [x] #13 Routing settings and fixed DNS settings are editable in the modal and classified as runtime-restart changes
- [x] #14 Dynamic dns.hosts remains safely file-only and is documented rather than flattened into invalid dotted paths
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Serialize routing and DNS into the edit session, excluding the dynamic dns.hosts map from scalar flattening. 2. Add DNS strategy choices and classify DNS/routing changes as active-runtime restart effects. 3. Add discovery, persistence, exclusion, and effect regression tests; update TUI documentation. 4. Run just fmt ci and finalize TASK-49.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented a global comma hotkey and sectioned searchable settings modal for operational config. The editor preserves comments, validates semantic constraints, writes atomically, masks secrets, supports env-backed secrets, and classifies changes as live, active-runtime restart, or daemon restart. Validation: just fmt ci passed (707 tests), and git diff --check passed.

Follow-up from hands-on TUI review: improve section hierarchy readability and repair Up/Down navigation in the Values pane.

Additional UI refinement: collapse third-level section nodes into grouped Values-pane subheaders (for example Runtime > Socks: General and Authentication).

Follow-up validation: section hierarchy, parent-page grouping, pane focus, and independent Values navigation are covered by state, keymap, helper, and render-level regression tests. just fmt ci passed with 712 tests; git diff --check passed.

Final visual refinement requested: display boolean values as glyphs in the Values pane.

Boolean Values-pane rendering now uses check/cross glyphs while preserving bool serialization and editing semantics. Render regression coverage verifies both states and the absence of true/false text. just fmt ci passed with 712 tests; git diff --check passed.

Final display refinement: replace raw empty-list brackets in the Values pane with a readable empty state.

Empty lists now render as none in the Values pane, without changing list edit input or TOML serialization. Regression coverage added; just fmt ci passed with 712 tests and git diff --check passed.

Display refinement: show zero as auto for the two test-worker concurrency settings whose documented semantics define 0 as automatic.

Zero now renders as auto for testing.concurrency and runtime.rotation.test_concurrency only. Regression coverage confirms runtime.mux.xudp_concurrency remains the literal 0 because its semantics differ. just fmt ci passed with 712 tests; git diff --check passed.

Values layout refinement: present fragment two-value ranges as indented min/max rows instead of comma-separated pairs.

Fragment packets, length, and interval pairs now render as one selectable field with indented min/max rows. Scrolling accounts for the selected three-row block, while editing and TOML serialization remain one list field. Render regression added; just fmt ci passed with 713 tests and git diff --check passed.

Scope extension approved: add routing and DNS fixed settings. dns.hosts remains file-only because dynamic keys require add/remove collection UX and hostname dots cannot safely use the scalar dotted-path editor.

Routing and fixed DNS settings are now serialized into the modal, with documented enum choices and RuntimeRestart effects. dns.hosts is explicitly excluded from edit-session serialization; persistence coverage verifies existing quoted host keys survive unrelated DNS/routing saves unchanged. Docs updated. just fmt ci passed with 714 tests; git diff --check passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added TUI management of operational config.toml settings, now including routing and fixed DNS options. The modal provides hierarchical grouped navigation and semantic value rendering; DNS/routing changes offer active-runtime restart. Dynamic dns.hosts remains file-only because it requires collection add/remove UX and quoted-key handling, and existing entries are preserved during saves. All writes remain atomic, validated, and comment-preserving. Verified with just fmt ci: 714 tests passed.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
