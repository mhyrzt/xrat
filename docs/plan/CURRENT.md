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

## Current Goal

Stage and execute Phase 4.5 implementation increments so that:

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

## Active Task Breakdown (Next)

1. **Daemon command scaffold**
   - add `xrat daemon start|status|stop` command shape in `src/cli/daemon.rs`
   - wire entrypoints into existing CLI root command tree
2. **IPC baseline**
   - add daemon server skeleton in `src/app/daemon/server.rs`
   - add protocol envelope (`protocol_version`, `code`, `message`, `payload`)
   - route `connect`/`disconnect`/`status` commands via IPC client path
3. **Supervisor ownership loop**
   - add `src/app/daemon/supervisor.rs` event queue + runtime owner state
   - persist macro transition records with reason code + origin fields
4. **Reattach and reconciliation hardening**
   - implement strict reattach checks (`pid`, executable, cmdline config path)
   - reject mismatches with explicit persisted reason code
5. **Replace bridge for area #5**
   - add make-before-break `RuntimeReplace` primitive
   - keep active runtime when candidate validation fails
6. **Targeted test pass**
   - IPC routing + daemon-unreachable guidance
   - reattach accept/reject regression coverage
   - unexpected process exit persistence
   - replace success/failure handoff safety

## Immediate Deliverables

To keep change risk controlled, first implementation batch should deliver:

- daemon command + IPC scaffold (no full rotation policy yet)
- supervisor process watch with failed-state persistence
- strict reattach verification path
- deterministic tests for new ownership contracts

## Success Criteria

Phase 4.5 staging is successful when:

- daemon ownership contracts are explicit and testable
- Phase 4 command semantics remain reusable behind IPC
- reattach behavior is deterministic and conservative
- area #5 receives stable replace/stop/handoff primitives
