# Current Work: Phase 4.5 Daemon Supervisor Planning

## Context

Phases 1 through 4 are now in place at the planning level, with Phase 4 focused
on managed runtime commands (`connect`, `disconnect`, `status`) and persisted
runtime session lifecycle.

The current planning focus has moved to Phase 4.5 so XRAT can transition from
command-driven reconciliation to an explicit background supervisor model.

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
  - defines supervisor state split (in-memory signals vs DB macro transitions)
  - defines conservative reattach policy
  - defines area #5 rotation bridge via make-before-break primitives

## Current Goal

Design and stage implementation for Phase 4.5 runtime supervision so that:

- one daemon owns runtime start/stop/reconcile decisions
- process exit is detected immediately while CLI is not running
- runtime transitions are persisted with stable reason taxonomy
- area #5 rotation can build on supervisor contracts without redesign

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

## Delivery Focus (Next)

1. Add daemon command bootstrap and supervisor skeleton.
2. Add local IPC server and CLI client wiring for connect/disconnect/status.
3. Add continuous process watch and failed-state persistence.
4. Add restart reconciliation and strict reattach verification.
5. Add make-before-break replace primitive for area #5 compatibility.
6. Add focused tests for ownership, reattach mismatch, and safe handoff.

## Success Criteria

Phase 4.5 planning is successful when:

- daemon ownership contracts are explicit and testable
- Phase 4 command semantics remain reusable behind IPC
- reattach behavior is deterministic and conservative
- area #5 receives stable replace/stop/handoff primitives
