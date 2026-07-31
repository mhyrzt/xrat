---
id: TASK-46
title: Investigate divergent CLI and TUI bulk test results
status: To Do
assignee: []
created_date: '2026-07-21 06:27'
updated_date: '2026-07-21 06:28'
labels:
  - bug
  - investigation
  - tui
  - cli
dependencies: []
references:
  - src/tui/run/tasks/test_batch.rs
  - src/tui/app/commands.rs
  - src/app/commands/test/settings/resolve.rs
  - src/app/commands/test/bulk/bulk_executor/bulk.rs
priority: medium
ordinal: 4000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Investigate why plain `xrat test` and the TUI `t + a` action can produce different results despite converging on the shared bulk-test executor. Known differences include config selection (CLI includes disabled configs unless `--enabled-only`; TUI selects enabled, non-deleted configs) and stage construction (TUI derives ICMP/real-delay/download flags from `runtime.rotation.test_stages` while always skipping TCP/upload; CLI follows enabled `[testing]` stages unless skip flags are passed). Determine whether parity is intended and identify the smallest corrective change or documentation update.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The mismatch is reproduced with the same database and configuration, and its root cause is documented.
- [ ] #2 A decision is recorded on whether the two entry points should be equivalent or intentionally different.
- [ ] #3 If parity is intended, the proposed fix includes a regression test covering equivalent selection and test settings.
- [ ] #4 The effective config set, stage flags, URLs, timeouts, and concurrency are compared for plain `xrat test` and TUI `t + a`.
<!-- AC:END -->





## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Acceptance criteria are satisfied or explicitly updated.
- [ ] #2 Relevant tests or checks were run and recorded in the task notes.
- [ ] #3 User-facing behavior changes are reflected in docs when applicable.
- [ ] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
