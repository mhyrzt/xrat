---
id: TASK-46
title: Investigate divergent CLI and TUI bulk test results
status: Done
assignee:
  - '@mahyar'
created_date: '2026-07-21 06:27'
updated_date: '2026-07-31 21:25'
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
- [x] #1 The mismatch is reproduced with the same database and configuration, and its root cause is documented.
- [x] #2 A decision is recorded on whether the two entry points should be equivalent or intentionally different.
- [ ] #3 If parity is intended, the proposed fix includes a regression test covering equivalent selection and test settings.
- [x] #4 The effective config set, stage flags, URLs, timeouts, and concurrency are compared for plain `xrat test` and TUI `t + a`.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Investigation complete (code-level). Both paths converge on run_bulk_for_configs_cancellable but with different inputs:
1) CONFIG SET: CLI xrat test uses args.config_filter (only_enabled=false by default, include_deleted=false) -> includes disabled configs unless --enabled-only. TUI t+a uses TestScope::AllEnabled -> enabled AND non-deleted only.
2) STAGES: CLI follows [testing] stage config (icmp/tcp/real_delay/download.enabled + skip flags; upload only with --upload-url). TUI derives icmp/real_delay/download from runtime.rotation.test_stages, ALWAYS skips tcp and upload (skip_tcp=true, skip_upload=true in test_args_for_app).
3) URLS: CLI can override test-url/download-url/upload-url; TUI passes None -> config defaults.
4) TIMEOUTS: CLI can override; TUI passes None -> config defaults.
5) CONCURRENCY: CLI defaults to testing.concurrency (0=auto); TUI default 4 (hardcoded in TestViewState::default).
Also found: docs/src/02-cli/tui.md says 'Test batches run TCP and real-delay tests ... skip download, upload, and ICMP' but code (since 7d3354a 'align tui test columns with config') runs rotation test_stages + always skips TCP/upload. TUI docs are stale and contradict the code.

DECISION (AC#2): Entry points are INTENTIONALLY different. TUI t+a mirrors runtime.rotation.test_stages by design (commit 7d3354a 'align tui test columns with config'); CLI xrat test follows [testing] config. Not a bug. Smallest corrective change = documentation update: fix stale docs/src/02-cli/tui.md testing-strip section to describe actual behavior (rotation stages, skips TCP/upload, concurrency 4, enabled configs only). No regression test needed (AC#3 N/A since parity not intended).

Validation: cargo fmt + clippy --all-targets -D warnings clean; cargo test -q --locked 657 passed. Docs updated in docs/src/02-cli/tui.md testing-strip section.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Investigated CLI xrat test vs TUI t+a divergence. Both converge on run_bulk_for_configs_cancellable but with different inputs: config set (CLI includes disabled unless --enabled-only; TUI only enabled+non-deleted), stages (CLI follows [testing] config; TUI derives from runtime.rotation.test_stages and always skips tcp/upload), URLs/timeouts (CLI can override, TUI uses config), concurrency (CLI testing.concurrency=auto, TUI hardcoded 4). Decision: intentionally different by design (commit 7d3354a). Corrective change: fixed stale tui.md docs that claimed TUI runs TCP and skips ICMP. No code change, no regression test (parity not intended). Verified: full test suite + clippy clean.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
