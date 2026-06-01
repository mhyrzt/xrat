# Engine Selection + Runtime Supervisor Parity Checklist (xray-knife -> xrat)

This checklist maps gap area **#4 Engine Selection Logic** and the runtime
ownership/supervision decisions linked from:

- `docs/validation/0_xray-knife_vs_xrat_gap_checklist.md`
- `docs/plan/PHASE_4.md`
- `docs/plan/PHASE_4p5.md`

---

## Scope and target behavior

Parity target for this phase bundle:

1. Managed runtime lifecycle parity (`connect`/`disconnect`/`status`) with
   DB-backed state.
2. Engine selection semantics parity decision (xray-focused vs multi-engine
   parity).
3. Process ownership, stale PID reconciliation, and failure reason persistence.
4. Supervisor/reattach policy decision capture (Phase 4.5 boundary).

Out of scope for this checklist:

- full rotating-proxy scheduler behavior (area #5),
- full scanner orchestration breadth (area #6),
- full sing-box runtime feature parity when runtime engine matrix is not yet
  enabled.

---

## Current state snapshot (xrat)

- Runtime commands exist:
  - `xrat connect <id>` (`src/cli/connect.rs`, `src/app/commands/connect.rs`)
  - `xrat disconnect` (`src/cli/disconnect.rs`,
    `src/app/commands/disconnect.rs`)
  - `xrat status` (`src/cli/status.rs`, `src/app/commands/status.rs`)
- Runtime lifecycle orchestration exists:
  - managed start/stop/status and stale reconciliation in
    `src/app/runtime_service.rs`.
- Persistence model exists:
  - `runtime_sessions` repository flow and lifecycle statuses (`starting`,
    `running`, `stopping`, `stopped`, `failed`).
- Runtime config generation exists:
  - runtime inbounds and launch path via `src/xray/config/*` +
    `src/xray/runtime.rs`.
- Engine wiring status:
  - runtime config can select binary path by configured engine name
    (`xray|v2ray|sing-box`) but runtime generation path remains xray-shaped.
- Supervisor status:
  - explicit daemon + supervisor loop exists with IPC routing and protocol
    version gating (`src/app/daemon/*`).

---

## A) Phase 4 managed runtime checklist

### `docs/plan/PHASE_4.md` alignment

Checklist:

- [x] `connect` command launches managed runtime from stored config id.
- [x] `disconnect` command stops managed runtime and updates persisted state.
- [x] `status` command reports runtime/session/config/inbound health view.
- [x] Runtime session lifecycle persisted in `runtime_sessions`.
- [x] Active config flag is updated on successful connect/disconnect flow.
- [x] Runtime supports configured local inbounds (SOCKS/HTTP/Shadowsocks).
- [x] Runtime startup failures persist short XRAT-owned `failure_reason`.
- [x] Stale PID/session reconciliation path exists in runtime service.
- [x] SQLite and PostgreSQL repository path support remains intact.
- [x] Connect rejects disabled/missing config ids with clear argument error.
- [x] Runtime supports replacement policy (`replace_active_session`) when
      connecting.
- [x] JSON output mode exists for connect/disconnect/status command UX.

Gap notes:

- **PARTIAL** parity vs xray-knife broader runtime ecosystem: managed single
  runtime flow is present, but rotation/health scheduler remains separate work.

---

## B) Engine selection logic checklist

### `docs/validation/0_xray-knife_vs_xrat_gap_checklist.md` section 4 alignment

Checklist:

- [x] Runtime engine name is configurable in app config.
- [x] Runtime binary path resolves from configured engine
      (`xray/v2ray/sing-box`).
- [x] Parse-time engine selector (`auto|xray|sing-box`) exists for diagnostics
      path.
- [ ] Add runtime engine abstraction layer equivalent to xray-knife core
      factory.
- [ ] Add runtime protocol-to-engine compatibility matrix for managed runtime.
- [ ] Add managed-runtime `auto` engine mode with deterministic selection rules.
- [ ] Add sing-box runtime config generation/execution path parity.

Gap notes:

- **DIFFERENT BY DESIGN / PARTIAL**: xrat currently keeps runtime orchestration
  xray-oriented while exposing engine path selection and parse-time auto logic.

---

## C) Phase 4.5 supervisor + reattach checklist

### `docs/plan/PHASE_4p5.md` alignment

Checklist:

- [x] Persist concise runtime `failure_reason` in `runtime_sessions`.
- [x] Reconcile stale sessions when runtime PID is no longer alive during
      command flow.
- [x] Surface runtime/session health via `status` including PID-running signals.
- [x] Add background supervisor/daemon that continuously watches runtime PID.
- [x] Immediately mark runtime `failed` without requiring next CLI command.
- [x] Reattach policy implementation for XRAT-owned running process after app
      restart.
- [x] Implement explicit daemon UX:
  - [x] `xrat daemon start`
  - [x] `xrat daemon status`
  - [x] `xrat daemon stop`
- [x] Add daemon IPC contract for runtime operations:
  - [x] `RuntimeConnect`
  - [x] `RuntimeDisconnect`
  - [x] `RuntimeStatus`
  - [x] `RuntimeReplace`
  - [x] `DaemonPing`
- [x] Ensure CLI runtime commands use daemon IPC when daemon is running.
- [x] Ensure daemon-unreachable runtime commands return explicit guidance and do
      not silently take direct runtime ownership.
- [x] Add request/response protocol versioning (`protocol_version`) for forward
      compatibility.
- [x] Persist transition reason taxonomy machine codes:
  - [x] `manual_connect`, `manual_disconnect`
  - [x] reattach accepted/rejected reason family
  - [x] unexpected exit / health-check failure
  - [x] replace started / validation failed / commit success / rollback
- [x] Persist transition origin (`cli|daemon|health_task|rotation_task`).
- [x] Add schema fields for daemon ownership traceability:
  - [x] `owner_kind`
  - [x] `owner_instance_id`
  - [x] `last_transition_reason_code`
  - [x] `last_transition_reason_detail`
- [x] Add candidate cooldown/failure fields needed by replace bridge:
  - [x] `cooldown_until`
  - [x] `last_failed_at`
  - [x] `last_failed_reason_code`
- [ ] Decide and implement auto-reconnect ownership boundary (supervisor now vs
      defer to later scheduler phase).
- [x] Add focused tests for:
  - [x] IPC routing + daemon-unreachable hints
  - [x] strict reattach accept/reject paths
  - [x] unexpected process exit persistence
  - [x] make-before-break replace success/failure safety

Gap notes:

- **MOSTLY COMPLETE FOR PHASE 4.5**: daemon ownership, IPC contract, reattach
  enforcement, transition taxonomy, cooldown/failure bridge fields, and core
  tests are implemented.
- Remaining item is product-boundary policy: whether auto-reconnect behavior is
  finalized in supervisor scope or deferred to scheduler phase.

---

## Suggested implementation order (remaining)

1. [ ] Finalize engine-direction decision for managed runtime: xray-focused for
       now vs explicit multi-engine parity target.
2. [ ] If multi-engine target approved, introduce runtime engine trait/factory
       and sing-box runtime config adapter.
3. [x] Add background supervisor command/process for continuous crash detection.
4. [x] Implement controlled reattach policy for XRAT-owned runtime artifacts.
5. [x] Add transition reason/origin persistence and daemon ownership fields.
6. [x] Implement make-before-break replace primitive (`RuntimeReplace`) for area
       #5 bridge.
7. [x] Add deterministic test coverage using runtime adapter fakes.

---

## Exit criteria for "Area #4 + Phase 4/4.5 review complete"

- [x] Phase 4 managed runtime baseline behavior is implemented and validated.
- [x] Engine-selection parity gaps are explicitly listed and scoped.
- [x] Phase 4.5 supervisor decisions are captured and implemented.
- [ ] Runtime engine abstraction parity decision is implemented (or documented
      non-goal).
- [x] Background supervisor/reattach policy is implemented.
- [x] IPC contract, reason taxonomy, and ownership schema additions are
      implemented and validated.

---

## Summary

- xrat Phase 4 baseline runtime management is in place and functional.
- Phase 4.5 supervisor/reattach baseline is implemented with daemon IPC and
  ownership/transition metadata persistence.
- Largest remaining parity pressure in this area is runtime multi-engine depth.

## Completion blockers

**Reviewed: 2026-06-01**
**Resolved: 2026-06-01**

The following items have been documented as product decisions or deferred features:

### 1. Runtime engine abstraction layer (Section B) - Documented as non-goal

xrat is xray/v2ray-focused for managed runtime. Multi-engine parity (sing-box) is deferred pending product decision.

### 2. Sing-box runtime config generation (Section B) - Documented as non-goal

Parse-time sing-box support exists for diagnostics (`--engine sing-box`). Managed runtime sing-box support is deferred pending product decision.

### 3. Auto-reconnect ownership boundary (Section C) - Documented as deferred

Auto-reconnect behavior is deferred to a later scheduler phase. Current supervisor handles crash detection and reattach policy.

### 4. Exit criterion - Documented as non-goal

"Runtime engine abstraction parity decision is implemented (or documented non-goal)" - Documented as non-goal: xrat is xray/v2ray-focused.
