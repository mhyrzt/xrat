---
id: TASK-71.1
title: TUI modal for editing config and subscription overrides
status: To Do
assignee: []
created_date: '2026-08-22 13:31'
labels: []
dependencies: []
parent_task_id: TASK-71
ordinal: 32000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Reuse the existing settings modal machinery (SettingsModalState/SettingsSession, src/tui/app/settings.rs and src/app/config/editor.rs) to open scoped override editors: hotkey on a selected row in the config list opens the config override modal, hotkey in the subscriptions view opens the subscription override modal. Each field is tri-state: inherit (default) or explicit value. Show the effective value and its source (global/sub/config) as a badge so users see what the parent layers contribute. Save persists only explicitly set fields as typed JSON to the corresponding overrides_json column; clearing a field returns it to inherit.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Config list hotkey opens override modal scoped to the selected config
- [ ] #2 Subscriptions view hotkey opens override modal scoped to the selected subscription
- [ ] #3 Fields default to inherit and saving writes only explicitly set fields
- [ ] #4 Effective value and its origin layer are visible for each field
- [ ] #5 Invalid input is rejected inline like the global settings modal
- [ ] #6 Changes take effect on next connect/test without requiring daemon restart
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
