---
id: TASK-15
title: 'Medium, P2: Make TCP a first-class test stage'
status: In Progress
assignee: []
created_date: '2026-07-05 14:43'
updated_date: '2026-07-06 12:09'
labels:
  - legacy-import
  - feature
dependencies: []
priority: medium
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
