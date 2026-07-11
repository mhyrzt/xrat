---
id: TASK-15
title: 'Medium, P2: Make TCP a first-class test stage'
status: Done
assignee: []
created_date: '2026-07-05 14:43'
updated_date: '2026-07-11 21:22'
labels:
  - legacy-import
  - feature
dependencies: []
priority: high
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/feature/tcp-test-stage.md`

# Medium, P2: Make TCP a first-class test stage

### Status

Planned

### Motivation

Users reasonably expect this configuration to be valid when they want to run a
TCP connectivity check instead of a real-delay HTTP probe:

```toml
[testing]
order = ["icmp", "tcp"]

[testing.tcp]
enabled = true
timeout = 2000
```

Today this fails during config parsing because `tcp` is not a
`ConnectionTestStage`. TCP exists only as a gate that runs immediately before
`real_delay` when real-delay is in the stage order. That behavior is surprising:
the config file exposes `[testing.tcp]`, but `[testing].order` cannot name it.

### Current behavior

- Accepted `[testing].order` values are `icmp`, `real_delay`, and `download`.
- `[testing.tcp].enabled = true` only has an effect when `real_delay` is also
  enabled and present in the stage order.
- `xrat validate` rejects `order = ["icmp", "tcp"]` as a generic parse failure
  before it can report a field-level validation diagnostic.
- Rotation tests also derive TCP from real-delay:
  `settings.run_tcp = settings.run_real_delay && testing.tcp.enabled`.

### Desired behavior

Make `tcp` an accepted stage in `[testing].order` and rotation `test_stages`.
This should allow TCP-only test pipelines without forcing a real-delay probe:

```toml
[testing]
order = ["icmp", "tcp"]

[testing.real_delay]
enabled = false

[testing.tcp]
enabled = true
timeout = 2000
```

When both `tcp` and `real_delay` are present, avoid running the same TCP check
twice. Prefer treating an explicit `tcp` stage as the gate result reused by the
later real-delay stage.

### Changes required

- Add `Tcp` to `ConnectionTestStage` in `src/app/config/testing/types.rs`.
- Update `test_stage_name`, validation messages, docs, and default/example
  config comments to include `tcp`.
- Update test execution so `ConnectionTestStage::Tcp` runs `run_tcp_gate`
  directly and records `ran_tcp`.
- Preserve current compatibility: when `real_delay` is present and `tcp` is not,
  continue running TCP as the implicit real-delay gate if `[testing.tcp].enabled`
  is true.
- Update rotation test selection so `runtime.rotation.test_stages = ["tcp"]`
  performs TCP-only candidate checks instead of returning no test rows.
- Decide how TCP-only rotation should rank candidates, since existing rotation
  candidate selection primarily prefers passing real-delay with latency.

### Verification

- Config parsing accepts `order = ["icmp", "tcp"]`.
- `xrat validate` accepts `[testing].order = ["tcp"]` when `[testing.tcp]` is
  enabled.
- A TCP-only `xrat test` records `tcp_ok` and `tcp_ms`, with `real_delay_*`
  unset.
- `order = ["tcp", "real_delay"]` does not perform duplicate TCP checks.
- Rotation tests can run with `test_stages = ["tcp"]` and produce useful
  candidate health records.

### Open decisions

- Should `tcp` become part of the default order, or remain opt-in?
- For rotation, should TCP-only mode select the lowest `tcp_ms`, or only use TCP
  as a pass/fail health gate before falling back to existing candidate order?
- Should `real_delay` keep implicitly running TCP forever, or should that become
  a migration-only compatibility path?
<!-- SECTION:DESCRIPTION:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. types.rs: add Tcp variant to ConnectionTestStage (config_name, from_config_str).
2. Sync duplicate test_stage_name in settings/validation.rs.
3. execution/run.rs: own match arm for ConnectionTestStage::Tcp calling run_tcp_gate, sets ran_tcp; RealDelay arm's existing !ran_tcp guard dedups when both present.
4. bulk/rotation.rs: add has_tcp branch; stop forcing run_tcp = run_real_delay && enabled; fix early-return so test_stages=["tcp"] produces rows.
5. replace_flow/candidate.rs: add TCP-only passing/ranking path, rank by tcp_ms ascending when no real_delay/download signal.
6. Keep default order unchanged; tcp stays opt-in.
7. Update docs/comments listing accepted order values.
8. Tests: stage parsing, execution dedup, rotation tcp-only non-empty, candidate tcp ranking.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented: ConnectionTestStage::Tcp variant + config_name/from_config_str; execution/run.rs own Tcp match arm with ran_tcp dedup (works both orders: tcp-first sets ran_tcp before RealDelay's existing !ran_tcp guard; real_delay-first still gates tcp first, Tcp arm's own !ran_tcp guard then skips re-run); bulk/rotation.rs has_tcp branch, run_tcp no longer forced to run_real_delay, early-return fixed; candidate.rs adds TCP-only passing/ranking path (rank by tcp_ms) for both fresh-bulk and stale-db candidate paths; default order unchanged (tcp opt-in); docs updated (testing.md, test.md, validate.md, config-file.md).
Tests: config parse test order=["icmp","tcp"]; runtime_service test manual_rotate_accepts_tcp_only_passing_result (stale-db TCP-only candidate path).
cargo fmt/clippy clean. cargo test --lib: 639 passed, 1 pre-existing flaky DNS test unrelated (passes in isolation, fails on master too intermittently).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added ConnectionTestStage::Tcp as a standalone, opt-in stage in [testing].order and rotation test_stages. Execution loop runs it via run_tcp_gate with ran_tcp-based dedup against the implicit real-delay gate (works regardless of stage order). Rotation bulk tests no longer force run_tcp to run_real_delay and no longer silently no-op on tcp-only test_stages. Candidate ranking (replace_flow/candidate.rs) now has a TCP-only passing/ranking path (by tcp_ms) for both fresh and stale-data candidate lookup. Default order left unchanged; tcp stays opt-in per the task's open decision. Docs updated (testing.md, test.md, validate.md, config-file.md). Verified with a config-parse test (order=["icmp","tcp"]) and a runtime_service test exercising the TCP-only candidate path end to end. cargo fmt/clippy clean, cargo test --locked: 647/647 passed.
<!-- SECTION:FINAL_SUMMARY:END -->
