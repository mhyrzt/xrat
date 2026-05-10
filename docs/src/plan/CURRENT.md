# Current Work: Phase 4.5 Daemon Supervisor Implementation Staging

## Context

Phases 1 through 4 are now in place at the planning level, with Phase 4 focused
on managed runtime commands (`connect`, `disconnect`, `status`) and persisted
runtime session lifecycle.

The work focus is now implementation staging for Phase 4.5 so XRAT can
transition from command-driven reconciliation to an explicit background
supervisor model.

## What Changed

Recent plan updates aligned `PHASE_4.md` and `PHASE_4p5.md` with
`docs/plan/DAEMON_GEMINI.md`.

- `docs/plan/PHASE_4.md`
  - explicitly marks daemon IPC ownership as out of Phase 4 scope
  - keeps background crash monitoring and make-before-break rotation out of
    Phase 4 scope
  - adds a clear Phase 4 -> 4.5 handoff contract for service/state continuity
- `docs/plan/PHASE_4p5.md`
  - adopts explicit `xrat daemon` ownership
  - defines CLI-as-IPC-client behavior
  - defines first-pass daemon IPC contract + response envelope
  - defines supervisor state split (in-memory signals vs DB macro transitions)
  - defines conservative reattach policy
  - defines transition reason taxonomy and origin fields
  - defines minimal schema additions for daemon ownership traceability
  - defines area #5 rotation bridge via make-before-break primitives
  - defines Phase 4.5 test matrix for IPC/reattach/replace safety
- implementation scaffolding is now started in code:
  - daemon CLI command surface (`xrat daemon start|status|stop`)
  - unix socket IPC server/client baseline (`runtime/daemon.sock`)
  - supervisor event bus (`mpsc` + `oneshot`) with routed request handling
  - routed daemon operations:
    - `DaemonPing`
    - `RuntimeStatus` (backed by `RuntimeService::status` with runtime/session
      summary payload)
    - `RuntimeConnect` (backed by `RuntimeService::connect`)
    - `RuntimeDisconnect` (backed by `RuntimeService::disconnect`)
  - CLI runtime command handlers now attempt daemon IPC first:
    - `connect` -> `RuntimeConnect`
    - `disconnect` -> `RuntimeDisconnect`
    - `status` -> `RuntimeStatus`
  - structured mutating command responses now return explicit `ok/code/message`
    with typed payloads on success

## Current Goal

Stage and execute Phase 4.5 implementation increments so that:

- one daemon owns runtime start/stop/reconcile decisions
- process exit is detected immediately while CLI is not running
- runtime transitions are persisted with stable reason taxonomy
- area #5 rotation can build on supervisor contracts without redesign

Current Phase 4.5 progress estimate: **72%** complete as of **2026-05-10**.

## Phase 4 Boundary (Now Explicit)

Phase 4 remains responsible for:

- managed runtime command semantics
- persisted runtime session lifecycle
- stale PID/session reconciliation on subsequent CLI commands

Phase 4 intentionally does not own:

- daemon IPC runtime ownership
- continuous background crash monitoring
- make-before-break rotation orchestration

## Phase 4.5 Target Architecture

### Process Model

- `xrat daemon` runs long-lived supervisor event loop
- CLI commands become local IPC clients when daemon is running
- one authoritative owner handles runtime lifecycle transitions

### Suggested Module Layout

- `src/cli/daemon.rs`
- `src/app/daemon/server.rs`
- `src/app/daemon/supervisor.rs`

### State Model

- volatile in-memory state for hot health/rotation signals
- persistent DB writes only on macro transitions
- avoid per-probe write amplification

### Reattach Policy

Startup verification should require all of the following before reattach:

- PID exists
- process executable matches expected runtime engine
- command line references XRAT-owned config path

If verification fails, session is marked stale/failed and active state is
reconciled.

## Active Task Breakdown (Next)

1. **Daemon command scaffold**

- add `xrat daemon start|status|stop` command shape in `src/cli/daemon.rs`
- wire entrypoints into existing CLI root command tree
- **status:** done

2. **IPC baseline**

- add daemon server skeleton in `src/app/daemon/server.rs`
- add protocol envelope (`protocol_version`, `code`, `message`, `payload`)
- route `connect`/`disconnect`/`status` commands via IPC client path
- **status:** in progress (connect/disconnect/status routes exist; fallback
  policy still transitional)

3. **Supervisor ownership loop**

- add `src/app/daemon/supervisor.rs` event queue + runtime owner state
- persist macro transition records with reason code + origin fields
- **status:** in progress (event loop + connect/status handlers wired;
  transition reason persistence still pending; disconnect handler now wired too)

4. **Reattach and reconciliation hardening**

- implement strict reattach checks (`pid`, executable, cmdline config path)
- reject mismatches with explicit persisted reason code
- **status:** in progress (strict checks and reject reasons landed; accepted
  ownership metadata schema still pending)

5. **Replace bridge for area #5**

- add make-before-break `RuntimeReplace` primitive
- keep active runtime when candidate validation fails
- **status:** in progress (replace IPC/supervisor/runtime contract landed with
  safety coverage; full make-before-break execution still pending)

6. **Targeted test pass**

- IPC routing + daemon-unreachable guidance
- reattach accept/reject regression coverage
- unexpected process exit persistence
- replace success/failure handoff safety
- **status:** in progress (CLI parse + focused command-path checks; broader
  daemon integration coverage pending)

## Current Gaps to Close

- runtime commands still allow direct fallback behavior when daemon is
  unreachable; Phase 4.5 target is explicit daemon guidance without silent
  ownership fallback.
- daemon status payload is currently a compact summary; full parity with local
  status output shape (inbound endpoint health, failure reason details) is still
  pending.
- transition reason taxonomy persistence (`reason_code`, `origin`,
  `reason_detail`) is not implemented in runtime session writes yet.
- strict reattach verification and restart reconciliation hardening are not
  fully implemented yet (ownership metadata schema still pending).

## Immediate Deliverables

To keep change risk controlled, first implementation batch should deliver:

- daemon command + IPC scaffold (no full rotation policy yet)
- supervisor process watch with failed-state persistence
- strict reattach verification path
- deterministic tests for new ownership contracts

## Phase 4.5 Execution Slices (Live Tracker)

Source of truth detail lives in `docs/src/plan/PHASE_4p5.md` under **Next Work
Slices (Implementation Order)**. This section mirrors status for day-to-day
tracking.

1. **Slice A - Daemon command correctness hardening**

- scope:
  - `src/app/commands/daemon.rs`
  - `src/app/daemon/client.rs`
  - `src/app/daemon/protocol.rs`
- focus:
  - make `xrat daemon start` detach/spawn-and-return
  - fail `daemon status|stop` when IPC reply has `ok=false`
  - enforce `protocol_version` compatibility gate
- status: mostly complete

2. **Slice B - Supervisor error semantics and failure visibility**

- scope:
  - `src/app/daemon/supervisor.rs`
  - `src/app/daemon/server.rs`
  - `src/app/runtime_service/`
- focus:
  - stop masking backend status errors as `"unknown"`
  - map status backend failures to structured `ok=false` IPC responses
  - persist unexpected process exits with `process_exit_unexpected`
- status: complete (status failure path)

3. **Slice C - Reattach verification and restart reconciliation**

- scope:
  - `src/app/runtime_service/` reattach verifier module
  - `src/app/daemon/supervisor.rs`
  - `src/db/repository/`
- focus:
  - enforce strict checks (`pid`, executable, cmdline/config ownership)
  - persist explicit reject reason codes and clear stale owner state
  - mark accepted reattach with daemon ownership metadata
- status: in progress

4. **Slice D - Replace primitive for Area #5 bridge**

- scope:
  - `src/app/daemon/supervisor.rs`
  - `src/app/runtime_service/`
  - `src/app/daemon/protocol.rs`
- focus:
  - add `RuntimeReplace` make-before-break flow
  - validate replacement readiness before ownership switch
  - persist rollback reason while keeping old runtime active on failure
- status: in progress

## Daily Progress Board (Phase 4.5)

Update this table during active implementation. Keep blocker text short and
actionable.

| Slice                           | Status      | Owner        | Last Updated | Blocker                                              |
| ------------------------------- | ----------- | ------------ | ------------ | ---------------------------------------------------- |
| A - Daemon command correctness  | Mostly complete | _unassigned_ | 2026-05-10   | Follow-up: daemon start output/UX polish only     |
| B - Supervisor error semantics  | Complete (status path) | _unassigned_ | 2026-05-10   | Optional: expand structured failures for other event types |
| C - Reattach and reconciliation | In progress | _unassigned_ | 2026-05-10   | Remaining: owner metadata schema fields |
| D - Replace primitive bridge    | In progress | _unassigned_ | 2026-05-10   | Remaining: true make-before-break handoff semantics |

## Phase 4.5 Closure Gates

Phase 4.5 should not be marked complete until all of these are passing:

- daemon command contracts:
  - detached start behavior
  - `status`/`stop` failure on `ok=false`
- protocol contract:
  - incompatible protocol version rejection
- supervisor contract:
  - backend status failures visible via structured daemon error response
  - unexpected runtime exit persisted with stable reason code
- reattach contract:
  - deterministic accept + reject path coverage (`pid_missing`, `exec_mismatch`,
    `cmdline_mismatch`)
- replace contract:
  - success handoff and keep-old-runtime rollback coverage

## Success Criteria

Phase 4.5 staging is successful when:

- daemon ownership contracts are explicit and testable
- Phase 4 command semantics remain reusable behind IPC
- reattach behavior is deterministic and conservative
- area #5 receives stable replace/stop/handoff primitives
