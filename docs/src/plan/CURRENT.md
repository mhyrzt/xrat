# Current Work: Phase 4.6 Auto-Rotating Proxy

## Context

Phase 4.5 daemon supervisor baseline is complete and now serves as the runtime
owner. Current implementation work is focused on Phase 4.6: a first dedicated
auto-rotating proxy layer that can test alternatives and perform safe
make-before-break replacement.

## What Landed So Far

- CLI surface added: `xrat proxy start|status|rotate|stop`.
- Daemon IPC contract extended for proxy control/status requests.
- Supervisor state extended with rotation controls:
  - `rotation_enabled`
  - `rotation_interval_secs`
  - `health_trigger_enabled`
  - `cooldown_secs`
  - `next_timer_epoch_secs`
  - `last_trigger`
  - `last_result`
  - `last_candidate_config_id`
  - `last_candidate_result`
  - `cooldown_active`
- Runtime config surface added under `[runtime.rotation]`:
  - `enabled = false`
  - `interval_secs = 1800`
  - `health_trigger_enabled = true`
  - `cooldown_secs = 300`
  - `test_concurrency = 0`
  - `test_stages = ["real_delay", "download"]`
- Supervisor startup now reads rotation defaults from app config.
- Health tick loop now drives automatic rotation hooks:
  - health-failure trigger path
  - timer-due trigger path
- Automatic candidate scoring now executes a dedicated persisted `rotation` bulk
  test run over eligible candidates before ranking.
- Candidate ranking policy implemented:
  - passing `real_delay_ok=true` candidates only
  - lowest `real_delay_ms`
  - tie-break by higher `download_mbps`
  - final tie-break by config id
- Manual force rotation (`proxy rotate --config-id`) continues to reuse existing
  `RuntimeReplace` safety flow.
- Rotation reason-code mapping now persisted for trigger start and failure
  outcomes:
  - `rotation_manual_started`
  - `rotation_timer_started`
  - `rotation_health_started`
  - `rotation_no_candidate`
  - `rotation_candidate_failed`
- Proxy status payload/output now includes last candidate id/result and
  `cooldown_active` summary.
- Supervisor test coverage expanded for rotation state behavior:
  - timer-trigger failure state tracking
  - health-trigger no-candidate state tracking
  - manual replace failure reason-code persistence
  - cooldown-suppressed health-tick state tracking
  - proxy-status payload field coverage for candidate/cooldown fields

## Current Goal

Finish Phase 4.6 parity so rotation behavior and status visibility match plan
contracts without introducing new persistence tables.

Progress estimate: **~93%** complete as of **2026-05-12**.

## Remaining Gaps

1. Close remaining supervisor branch coverage for successful timer/manual
   replacement paths (failure/no-candidate semantics are now covered).
2. Optional: add JSON output path for `proxy status`.

## Immediate Next Slice

- Test-first pass on supervisor/runtime rotation branches:
  - timer-trigger selection and scheduling updates
  - health-trigger replacement fallback behavior
  - manual override/cooldown branch coverage
- Then close remaining coverage gaps and optional status JSON output.
