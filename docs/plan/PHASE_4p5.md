# Phase 4.5 Runtime Supervisor And Reattach Policy

## Goal

Define the behavior that needs a background runtime supervisor, daemon, or
watcher instead of the current command-driven lifecycle checks.

Phase 4 keeps XRAT as a CLI that starts Xray, records the runtime session, and
reconciles state when the user next runs `connect`, `disconnect`, or `status`.
That is enough for a first managed runtime, but it does not continuously watch
for crashes after the CLI exits.

## Validation Link

Related parity checklist source:

- `docs/validation/0_xray-knife_vs_xrat_gap_checklist.md`
  - section **4) Engine Selection Logic (xray vs sing-box)**
  - section **5) Auto-Rotating Proxy**
- `docs/validation/4_engine_runtime_supervisor_parity_checklist.md`
  - section **C) Phase 4.5 supervisor + reattach checklist**
- `docs/validation/5_auto_rotating_proxy_parity_checklist.md`
  - rotation trigger model (timer/manual/health-failure)
  - handoff safety expectations (start replacement then drain/stop old)
  - blacklist/cooldown policy inputs for later scheduler phases

Use those checklist sections as external parity pressure when deciding daemon
model, reattach policy, and runtime failure reconciliation behavior.

## Scope

Phase 4.5 should cover:

- deciding whether XRAT should run a background supervisor process
- detecting Xray/V2Ray exits immediately instead of waiting for the next CLI
  command
- optionally reattaching to an already-running managed process after app restart
- surfacing runtime failure reasons without relying only on Xray/V2Ray log files
- defining whether automatic reconnect belongs in the supervisor or a later
  health-check phase

Phase 4.5 should additionally provide process-ownership contracts that area #5
(auto-rotating proxy) can rely on:

- one authoritative owner for start/stop/reconcile transitions
- atomic active-session handoff semantics to avoid orphaned runtime processes
- clear transition reasons that rotation logic can consume (`process exited`,
  `health-check failed`, `manual rotate`, `timer rotate`)

## Background Watcher Work

A daemon or watcher would be responsible for:

- monitoring the saved runtime PID while the proxy is active
- marking `runtime_sessions.status` as `failed` when the process exits
- storing a short XRAT-owned `failure_reason` such as `process exited`,
  `startup timeout`, or `inbound closed`
- clearing `configs.is_active` when the managed runtime is no longer alive
- optionally tailing or linking generated stdout/stderr log paths for
  diagnostics

This is intentionally deferred from Phase 4 because the current CLI process
exits after `connect`, so continuous monitoring requires a new long-lived
process model.

For parity alignment with area #5, the watcher should expose minimal hooks that
future rotation logic can reuse:

- runtime health signal stream (or equivalent event polling contract)
- deterministic transition writes to `runtime_sessions`
- safe replace/stop primitives that keep `configs.is_active` and session status
  synchronized

## Open Decisions

- should the watcher be an explicit `xrat daemon` command or an implicit
  background process started by `connect`?
- should `status` only report persisted state from the daemon, or should it keep
  doing local PID/inbound checks as a fallback?
- should reattach be supported for any matching PID, or only for processes with
  XRAT-owned config/log paths?
- should automatic reconnect be part of Phase 4.5 or a later runtime-health
  phase?
- should Phase 4.5 include a minimal manual rotate trigger contract so area #5
  can be layered without redesigning supervisor ownership?

## Suggested Delivery Order (Phase 4.5 with area #5 compatibility)

1. Choose daemon ownership model (`xrat daemon` explicit command is preferred).
2. Implement continuous PID watch and immediate failed-state persistence.
3. Implement restart-time reconciliation + XRAT-owned reattach policy.
4. Define stable runtime transition reason taxonomy for future rotation events.
5. Expose safe replace/stop service primitives that later `proxy` rotation code
   can call directly.

## Exit Criteria

- daemon/watcher updates runtime failure state without waiting for next CLI
  command
- active config/session drift is reconciled automatically on crash/exit
- reattach policy is implemented or explicitly documented as non-goal
- supervisor contracts needed by
  `docs/validation/5_auto_rotating_proxy_parity_checklist.md` are defined and
  tested
