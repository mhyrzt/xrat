# High, P1: Runtime rotation and daemon restart behavior

### Status

Draft

### Scope

Runtime lifecycle, rotation candidate selection, and event emission rate.

### Items

- `xrat daemon restart` should reattach previous connection session or select
  best valid connection deterministically.
- `xrat rotate now` sometimes selects invalid configs with missing/unusable
  delay results.
- Rotation logs show very frequent `rotation_bulk_advanced` bursts; validate
  whether scheduler/run-loop behavior is expected.

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
