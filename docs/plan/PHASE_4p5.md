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

## Open Decisions

- should the watcher be an explicit `xrat daemon` command or an implicit
  background process started by `connect`?
- should `status` only report persisted state from the daemon, or should it keep
  doing local PID/inbound checks as a fallback?
- should reattach be supported for any matching PID, or only for processes with
  XRAT-owned config/log paths?
- should automatic reconnect be part of Phase 4.5 or a later runtime-health
  phase?
