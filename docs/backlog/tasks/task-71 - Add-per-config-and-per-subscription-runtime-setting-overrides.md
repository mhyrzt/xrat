---
id: TASK-71
title: Add per-config and per-subscription runtime setting overrides
status: To Do
assignee: []
created_date: '2026-08-22 13:31'
updated_date: '2026-08-22 13:32'
labels: []
dependencies: []
references:
  - >-
    docs/backlog/drafts/draft-2 -
    Hard-P3-Per-config-Mux-Fragment-tuning-optimizer.md
ordinal: 31000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Introduce a three-layer settings resolution chain: global config.toml [runtime] defaults, overridable per subscription, overridable per config. Effective value = global <- subscription.overrides_json <- config.overrides_json, where None at each layer means inherit from the parent layer. Extends DRAFT-2 Phase 1 (which proposed only per-config -> global) to include the subscription layer. Overrides are typed structs serialized as JSON in new dedicated columns; do NOT reuse configs.extensions_json (holds protocol extensions pbk/sid/fp). Do not persist full root-level Xray JSON.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 New ordered migrations on sqlite and postgres add overrides_json columns to subscriptions and configs tables
- [ ] #2 Override types use Option fields so unset values inherit from the parent layer
- [ ] #3 A merge function resolves effective RuntimeSettings as global then subscription then config overrides
- [ ] #4 Merged settings drive build_xray_gen_options, the managed launch path, and ResolvedTestSettings.gen_options probe path
- [ ] #5 DB round-trip tests cover SQLite and Postgres for both subscription and config overrides
- [ ] #6 Resolution tests prove precedence: config override beats subscription override beats global default, and unset falls back correctly
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
