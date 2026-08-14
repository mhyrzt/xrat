---
id: TASK-56
title: 'tui: add contextual help for settings fields'
status: Done
assignee:
  - '@codex'
created_date: '2026-08-14 18:39'
updated_date: '2026-08-14 18:50'
labels:
  - tui
  - config
dependencies: []
ordinal: 16000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a persistent contextual Help pane to the config settings modal so every exposed field explains its purpose, accepted values or format, a safe TOML assignment example, and when the change takes effect.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Help pane follows the currently selected setting without adding a focus mode
- [x] #2 Every setting exposed by ConfigEditSession has a concise description and TOML assignment example
- [x] #3 Possible values come from actual setting kinds where authoritative and otherwise show field-specific constraints
- [x] #4 Help communicates live, runtime-restart, or daemon-restart impact
- [x] #5 Secret values never appear in help content or examples
- [x] #6 Tests cover metadata completeness, navigation updates, representative types, and compact terminal rendering
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add centralized per-path help metadata and attach it to editable settings with exhaustive coverage tests. 2. Derive boolean/enum possible values from SettingKind and use field-specific format constraints for remaining types. 3. Add a persistent full-width Help pane with description, values, TOML example, and application effect. 4. Add rendering/navigation/secret/compact-layout tests, update TUI docs, and run just fmt ci.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented a centralized help registry covering every currently exposed setting. Editable settings now derive boolean/enum possible values from SettingKind, carry safe TOML examples, and expose effect explanations. Added the persistent Help pane plus compact-terminal, navigation, completeness, and secret-safety regression coverage; focused settings tests pass (35).

Final validation: just fmt ci passed with 716 tests; git diff --check passed. The Help pane renders at 80x24 and normal sizes, follows real SettingsMove navigation, derives enum/boolean values from SettingKind, and preserves secret masking.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added a persistent contextual Help pane to the TUI settings modal. Every exposed setting now has curated purpose text, accepted values or format, a safe TOML assignment example, and live/runtime/daemon application guidance. Metadata coverage is enforced against ConfigEditSession, enum and boolean choices derive from editing metadata, and secret values remain masked. Updated TUI docs and verified with just fmt ci: 716 tests passed.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
