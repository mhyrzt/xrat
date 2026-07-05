---
id: DRAFT-2
title: 'Hard, P3: Per-config Mux/Fragment tuning optimizer'
status: Draft
assignee: []
created_date: '2026-07-05 14:43'
labels:
  - legacy-import
  - feature
dependencies: []
priority: low
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/feature/runtime-tuning-optimizer.md`

# Hard, P3: Per-config Mux/Fragment tuning optimizer

### Status

Draft

### Goal

For a given config, automatically search Mux and fragmentation parameter sets,
measure each, and persist the best per-config tuning (including whether each
feature should be enabled) in the database. Generated runtime/probe configs then
prefer the stored per-config tuning over the global `[runtime]` defaults.

### Background

Global tuning shipped first:
[runtime-mux-settings.md](runtime-mux-settings.md),
[runtime-fragmentation-settings.md](runtime-fragmentation-settings.md). Those are
`[runtime.mux]` / `[runtime.fragment]` sections applied globally to the proxy
outbound via `apply_runtime_tuning` (`src/xray/config/tuning.rs`), and the probe
runners already accept `&XrayGenOptions`.

This feature adds two things that do not exist yet:

1. Per-config tuning **storage and resolution** (overrides the global default).
2. A **tuner** that measures candidate parameter sets and picks the best.

### Prerequisite 1 — per-config tuning storage

Currently tuning is global only. The optimizer requires persisted per-config
overrides. This intentionally reverses the current "tuning is a global runtime
option, do not store per-node" stance and must be a deliberate decision.

- New table (or columns), e.g. `config_tuning`:
  - `config_id` (FK), `mux_enabled`, `mux_concurrency`, `mux_xudp_concurrency`,
    `mux_xudp_proxy_udp443`, `frag_enabled`, `frag_packets_mode`, `frag_packets`,
    `frag_length`, `frag_interval`, `score`, `measured_at`.
  - Add an ordered migration under `migrations/sqlite/` and
    `migrations/postgres/` (do not edit released migrations).
- Resolution order at generation time: **per-config override → global
  `[runtime]` default**. Today `build_xray_gen_options`
  (`src/app/runtime_tuning.rs`) reads only `RuntimeSettings`; it must also accept
  a per-config override and merge.
- Thread the override into both the runtime launch path
  (`src/app/runtime_service/launch.rs`) and the probe path
  (`ResolvedTestSettings.gen_options`).

### Prerequisite 2 — the tuner

Reuse the existing probe harness. The probe runners already take
`&XrayGenOptions`, so a candidate is just a different options value.

1. Build a candidate set (see cost — prefer presets, not a full grid).
2. For each candidate: `generate_probe_config_with_options` → run download +
   real-delay probes, repeated for several trials.
3. Score each candidate from aggregated trial stats.
4. Persist the best candidate's tuning to the config's row.

New command, e.g. `xrat optimize <config-id>` (handler under
`src/app/commands/`, CLI under `src/cli/`), writing through a repository method
in `src/db/`.

### Catch 1 — measurement noise and cost

- A single probe pass is noisy; Mux/fragment effects are path- and
  time-dependent. Use **repeated trials** with median/confidence, or the tuner
  overfits to transient conditions.
- The parameter space is combinatorial: Mux concurrency × fragment
  (mode × length range × interval range) × trials × per-config = many Xray
  spawns. Use **coarse presets** (disabled / default / aggressive) or bisection,
  bounded by time, not an exhaustive sweep.

### Catch 2 — objective mismatch (the hard limit)

- **Mux**: Xray docs say it helps many short-lived requests but commonly hurts
  throughput. "Best" depends on the real **workload**, which a synthetic 50MB
  download probe does not represent. A throughput-only objective will almost
  always pick `mux = disabled`. Mux optimization is near-useless without a
  workload model.
- **Fragment**: this is **circumvention, not performance**. Its value is binary
  reachability under blocking. A probe from an **unblocked** vantage cannot
  measure that benefit, so from a free network the tuner sees no gain and
  disables it. Fragment tuning is only meaningful when probing from the actual
  censored network.

Net expectation: a synthetic tuner reliably learns "fragment on/off from the
target network" and "mux off most of the time". Set expectations accordingly.

### Proposed scope

- Phase 1: per-config tuning storage + resolution (override → global default),
  manual set/clear, no search yet. This is independently useful.
- Phase 2: coarse preset tuner with repeated trials and an explicit objective
  weight (throughput vs latency vs success), persisting the winner.
- Defer: fine-grained grids, workload modeling, sing-box mapping.

### Engine scope

Xray only. Managed sing-box currently supports hy2 only and has no tuning
mapping; do not block this on sing-box.

### Verification

- DB tests: per-config tuning round-trips on SQLite and Postgres; resolution
  prefers the override and falls back to the global default.
- Generation tests: stored override changes generated outbound options.
- Tuner test: deterministic candidate scoring picks the expected winner from
  injected/mocked probe results (no live network in unit tests).
- Manual: `xrat optimize <config-id>` on a real config, confirm a persisted row
  and that `connect` uses it.

### Open decisions

- Objective: single weighted score, or separate latency/throughput/success
  targets the user selects?
- Candidate generation: fixed presets, configurable grid, or adaptive search?
- Re-tuning policy: one-shot, on demand, or periodic (and does rotation trigger
  it)?
- Should per-config tuning be importable/exportable, or strictly local measured
  state?
<!-- SECTION:DESCRIPTION:END -->
