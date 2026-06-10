# High, P1: Runtime rotation and daemon restart behavior

### Status

Draft

### Scope

Runtime lifecycle, rotation candidate selection, and event emission rate.

### Items

- ~~`xrat daemon restart` should reattach previous connection session or select
  best valid connection deterministically.~~ Resolved: daemon start runs
  `reconcile_reattach_on_daemon_start`, which validates the persisted session's
  PID/exec/cmdline and either keeps it, relaunches the same config on a stale
  PID (reboot case), or fails the session. Auto-selecting a _different_
  best-valid config on restart was intentionally not added (surprising to
  auto-connect a config the user did not pick).
- ~~`xrat rotate now` sometimes selects invalid configs with missing/unusable
  delay results.~~ Resolved: manual rotation no longer falls back to the
  lowest-id eligible config. It selects only configs with a passing real-delay
  result and otherwise fails with a clear message pointing at
  `xrat test`/`xrat scan`.
- ~~Rotation logs show very frequent `rotation_bulk_advanced` bursts; validate
  whether scheduler/run-loop behavior is expected.~~ Resolved: per-candidate
  `rotation_bulk_advanced` events were a cross-process progress bus polled by
  the `rotate` CLI but persisted as durable log rows. The per-candidate emit was
  removed; only bounded `rotation_bulk_started`/`rotation_bulk_finished` events
  and the `test_run` summary remain.

### Possible root causes

- Restart path likely restores process state but does not fully restore/persist
  prior active config selection and health snapshot.
- Candidate filtering for rotate-now may not enforce delay availability/validity
  before ranking.
- Rotation event logging may emit per-candidate step events at info level
  without aggregation/throttling, making normal runs look suspiciously noisy.

### Changes required

- Define restart contract: restore prior session if valid, otherwise fallback to
  best candidate.
- Tighten rotation preconditions to exclude configs without valid required probe
  results.
- Review rotation scheduler/retry loops and event severity/aggregation policy.

### Verification

- Regression tests for restart with/without persisted active session.
- Rotation tests proving invalid/no-delay configs are excluded.
- Runtime/log test ensuring bulk rotation emits expected bounded progress
  events.
