# Phase 4.6 Auto-Rotating Proxy

## Summary

Phase 4.6 builds the first dedicated auto-rotating proxy layer on top of the
Phase 4.5 daemon supervisor. The goal is to let XRAT keep one active proxy
runtime, test alternatives, select a better candidate, and rotate safely through
the existing make-before-break runtime replacement primitive.

This phase is intentionally focused on rotation. Runtime engine abstraction,
sing-box managed runtime parity, scanner parity, and detailed rotation event
history stay outside this phase.

## Key Changes

- Add `xrat proxy start|status|rotate|stop`.
  - `start` enables daemon-owned rotation scheduling.
  - `status` reports active config, last trigger, last candidate/result,
    cooldown summary, and next timer rotation.
  - `rotate` performs manual force rotation; `--config <id>` may target a
    specific config.
  - `stop` disables rotation scheduling without necessarily stopping the active
    runtime.
- Reuse Phase 4.5 daemon and supervisor ownership.
  - Add supervisor rotation state and events for timer, manual, and
    health-failure triggers.
  - Keep runtime lifecycle changes routed through existing `RuntimeReplace`.
  - Fix manual cooldown policy: automatic timer/health replacement respects
    cooldown; manual force rotation may override cooldown.
- Reuse the existing bulk test pipeline for candidate scoring.
  - Candidate pool: enabled configs excluding the active config for automatic
    rotation.
  - Ranking: passing configs by lowest `real_delay_ms`; tie-break by higher
    `download_mbps` when present; final tie-break by config id.
  - If no tested candidate passes, keep the current runtime active and
    persist/report a no-eligible-candidate result.
- Avoid new persistence tables in v1.
  - Use existing `runtime_sessions` transition fields and cooldown/failure
    fields.
  - Use reason codes such as `rotation_timer_started`,
    `rotation_manual_started`, `rotation_health_started`,
    `rotation_no_candidate`, `rotation_candidate_failed`, and existing replace
    success/failure codes.
  - Defer detailed rotation event history to a later phase.
- Add runtime config settings under `[runtime.rotation]`.
  - `enabled = false`
  - `interval_secs = 1800`
  - `health_trigger_enabled = true`
  - `cooldown_secs = 300`
  - `test_concurrency = 0`
  - `test_stages = ["real_delay", "download"]`

## Test Plan

- CLI parsing tests for `proxy start|status|rotate|stop`, including
  `rotate --config <id>` and JSON/status output if added.
- Supervisor tests for:
  - timer trigger selects best eligible candidate by real-delay-first ranking,
  - health trigger respects cooldown and keeps old runtime when no replacement
    exists,
  - manual force rotate overrides cooldown,
  - explicit manual config id validates missing/disabled config errors.
- Runtime safety tests:
  - successful rotation stages new runtime before stopping old runtime,
  - candidate test failure and replacement failure keep old runtime active,
  - cooldown metadata suppresses repeated automatic retries.
- End-to-end command tests around daemon IPC response mapping and
  daemon-unreachable guidance.

## Assumptions

- Save path is `docs/src/plan/PHASE_4p6_AUTO_ROTATING_PROXY.md`.
- Scope is rotation only; engine abstraction and sing-box runtime parity stay
  outside this plan.
- No dedicated rotation events table is added for v1.
- Timer rotation defaults to 30 minutes when rotation is enabled.
- Health-failure rotation never disconnects the current runtime just because no
  replacement is available.
