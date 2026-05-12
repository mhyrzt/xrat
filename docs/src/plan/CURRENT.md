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

## Current Goal

Finish Phase 4.6 parity so rotation behavior and status visibility match plan
contracts without introducing new persistence tables.

Progress estimate: **~80%** complete as of **2026-05-12**.

## Remaining Gaps

1. Persist and align dedicated rotation reason codes across start/attempt/fail
   outcomes (`rotation_timer_started`, `rotation_health_started`,
   `rotation_manual_started`, `rotation_no_candidate`,
   `rotation_candidate_failed`).
2. Expand `proxy status` payload/output with explicit last candidate id/result
   and cooldown summary details.
3. Add targeted supervisor tests for timer/health/manual branches and cooldown
   semantics under the new rotation state model.
4. Optional: add JSON output path for `proxy status`.

## Immediate Next Slice

- Test-first pass on supervisor/runtime rotation branches:
  - timer-trigger selection and scheduling updates
  - health-trigger replacement fallback behavior
  - manual override/cooldown branch coverage
- Then align reason-code persistence and status reporting fields to close
  remaining checklist items.
