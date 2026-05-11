# Phase 4.5 Runtime Supervisor, Daemon IPC, and Reattach Policy

## Goal

Introduce an explicit long-lived XRAT daemon that owns runtime process
lifecycle, watches failures in real time, and provides stable ownership
contracts for area #5 (auto-rotating proxy).

Phase 4 keeps XRAT command-driven (`connect`/`disconnect`/`status`) with
reconciliation on the next CLI call. Phase 4.5 moves runtime ownership to a
single background supervisor so crash detection, rotation triggers, and session
state updates do not depend on user-initiated commands.

## Progress Report (2026-05-11)

Estimated completion: **100%**

Phase 4.5 scope in this document is complete. Remaining work is follow-up polish
and next-phase scheduler behavior tracked outside this phase.

### Completed

- Added `xrat daemon start|status|stop` CLI surface and parser/help coverage.
- Added local Unix socket IPC server/client with request envelope and
  `protocol_version` field.
- Added supervisor event channel for `ping`, `status`, `connect`, `disconnect`,
  and `shutdown`.
- Wired `connect`/`disconnect`/`status` commands to prefer daemon IPC when
  reachable.
- Added daemon server tests for startup conflict and shutdown behavior.
- Runtime `connect`/`disconnect`/`status` no longer silently fall back to direct
  CLI ownership when daemon IPC is unreachable; they now return explicit
  `xrat daemon start` guidance.
- Added command-level regression coverage that asserts daemon-unreachable
  guidance for `connect`, `disconnect`, and `status`.
- Added schema groundwork for ownership/transition metadata on
  `runtime_sessions` (`owner_kind`, `owner_instance_id`,
  `last_transition_reason_code`, `last_transition_reason_detail`) with
  cross-backend migrations and model/repository mapping.
- Wired daemon ownership + transition metadata writes:
  - daemon boot now has an instance id
  - reattach accept/reject persists owner + reason metadata
  - daemon-driven connect/disconnect/replace-success persists reason codes
    (`manual_connect`, `manual_disconnect`, `replace_commit_success`)
  - daemon-driven connect/disconnect/replace-success now persists detail text
    alongside reason codes for diagnostics
- Extended transition reason writes in runtime lifecycle paths:
  - stale runtime reconciliation now persists `process_exit_unexpected`
  - replace flow now persists `replace_started`, `replace_validation_failed`,
    and `replace_rollback_keep_old`

### Completed (late-slice items)

- Runtime supervision now surfaces runtime backend status failures through
  structured daemon error envelopes (`ok=false`, `code=internal_error`) instead
  of reporting `"unknown"` as success.
- Daemon command handlers now treat `ok=false` as command failure for daemon
  `status` and `stop`.
- Server now enforces `protocol_version` compatibility and returns structured
  rejection for mismatches.
- Startup reattach reconciliation is wired into supervisor boot with strict
  checks (`pid`, executable identity, cmdline config-path ownership) and
  explicit reject reason codes persisted via runtime session failure reason.
- Added `RuntimeReplace` request/response contract across daemon IPC, supervisor
  event handling, and runtime service API.
- Added focused replace safety coverage:
  - runtime service rejects replace without running session
  - replacement validation failure keeps old runtime active
  - replacement spawn failure keeps old runtime active
  - daemon IPC replace success/error response mapping tests
- Added transition taxonomy schema extension for origin persistence:
  - `runtime_sessions.last_transition_origin` migration for SQLite/Postgres
  - model/repository/database mapping and write-path wiring
- Runtime replace flow now stages candidate runtime on alternate local inbound
  ports, marks candidate session running, switches active config, then stops old
  runtime session (make-before-break baseline).
- Added direct-runtime metadata parity for CLI-owned runtime service paths:
  - direct `connect` now persists `manual_connect` + detail + `origin=cli`
    - `owner_kind=cli`
  - direct `disconnect` now persists `manual_disconnect` + detail +
    `origin=cli` + `owner_kind=cli`
- Added deterministic replace success-path handoff assertion coverage:
  - validates new running session transition metadata includes
    `replace_commit_success` and origin/detail fields
- Stale session reconciliation now preserves ownership-aware transition origin:
  - `process_exit_unexpected` / stale-stop metadata now uses session
    `owner_kind` when present (for example `cli`) instead of always writing
    daemon origin.
- Added scheduler bridge schema fields and wiring for future rotation phases:
  - `runtime_sessions.cooldown_until`
  - `runtime_sessions.last_failed_at`
  - `runtime_sessions.last_failed_reason_code`
  - SQLite/Postgres migrations + model/repository row mapping
- Added initial failure-tracking write paths for scheduler bridge data:
  - stale reconcile on dead running process now writes `last_failed_at` +
    `last_failed_reason_code=process_exit_unexpected`
  - replace candidate spawn failure now writes `last_failed_at` +
    `last_failed_reason_code=replace_validation_failed`
- Added timer-driven supervisor health tick scaffold:
  - supervisor loop runs periodic health ticks (15s interval)
  - unreachable runtime inbound health now persists `health_check_failed`
    transition/failure metadata
  - cooldown bridge value `cooldown_until` is set to now + 300s
- Added cooldown-aware replace candidate selection for rotation triggers:
  - timer/health-triggered replace now selects from enabled alternatives that
    are not on active cooldown
  - manual replace keeps explicit behavior (defaulting to active config restart
    when no candidate id is supplied)
  - when all alternatives are cooling down, replace returns explicit invalid
    argument error
- Added suppression guard for repeated health-failure writes:
  - daemon health tick does not rewrite `health_check_failed` metadata while an
    existing cooldown window is still active
- Added cooldown lifecycle replacement regression coverage:
  - health-triggered replace rejects when all alternatives are cooling down
  - health-triggered replace re-selects an alternative after cooldown expiry
- Added daemon-level supervisor integration coverage for cooldown chain:
  - direct `HealthTick` event handling writes health-failure cooldown metadata
  - subsequent `RuntimeReplace { trigger = health_check_failed }` rejects cooled
    candidate alternatives when ineligible

### Not Started (outside Phase 4.5)

- Full scheduler policy tuning and ranking behavior for area #5 remains in the
  auto-rotating proxy phase.

### Review Findings Snapshot (current diff)

These findings came from reviewing the current daemon-related changes and should
be resolved before marking Phase 4.5 complete:

- Closed: `xrat daemon start` now detaches by spawning internal daemon serve
  mode and returning after socket readiness check.
  - `src/app/commands/daemon.rs`
  - `src/cli/daemon.rs`
- Closed: daemon `status`/`stop` now fail when daemon replies `ok=false`.
  - `src/app/commands/daemon.rs`
- Closed: supervisor runtime status no longer masks backend failures.
  - `src/app/daemon/supervisor.rs`
  - `src/app/daemon/server.rs`
- Closed: server enforces `protocol_version` gate with mismatch rejection test.
  - `src/app/daemon/server.rs`
- Closed: reattach coverage now includes explicit `cmdline_mismatch` reject
  regression.
  - `src/app/runtime_service/tests/reattach_cases.rs`
- Remaining: no blocker items in Phase 4.5 scope; status payload parity
  enrichment can proceed as follow-up polish alongside next phase work.

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

## Architecture Decision

Phase 4.5 should standardize on:

- explicit daemon command: `xrat daemon`
- CLI as IPC client instead of direct process owner for runtime operations
- one authoritative supervisor loop for start/stop/rotate/reconcile transitions

Recommended process split:

- `src/cli/daemon.rs`
  - daemon command flags and lifecycle entrypoint
- `src/app/daemon/server.rs`
  - IPC listener and request routing
- `src/app/daemon/supervisor.rs`
  - runtime event loop and process ownership

IPC transport options:

- Unix domain socket (`tokio::net::UnixListener`) on Unix-like platforms
- localhost HTTP/JSON bound only to `127.0.0.1` for cross-platform fallback

## Scope

Phase 4.5 should cover:

- daemonized ownership of Xray/V2Ray runtime processes
- immediate failure detection while CLI is not running
- restart-time reconciliation and controlled reattach
- persisted transition reasons suitable for future rotation policy
- safe replace/stop primitives that area #5 can call

Phase 4.5 should not yet cover:

- full rotation scheduler policy tuning (cooldown windows, ranking weights,
  adaptive scoring)
- external remote control API surface beyond local IPC
- multi-instance cluster orchestration

## Supervisor State Model

Use in-memory volatile state for hot runtime signals plus SQLite/PostgreSQL for
macro transitions.

Volatile in-memory state (for example `Arc<RwLock<SupervisorState>>`):

- active runtime PID/session id
- rolling health metrics (latency window, consecutive failures)
- timestamp of last successful rotation/handoff
- in-flight transition marker (`starting`, `rotating`, `stopping`)

Persistent DB writes (only on macro transitions):

- proxy selected/activated
- runtime process started/running/failed/stopped
- candidate marked failed and cooldown/blacklist-related fields updated

Do not write to DB for every probe ping/health sample.

## Reattach Policy

Reattach should be XRAT-owned and conservative.

On daemon startup:

1. Read XRAT PID/session metadata (pid file + runtime session row).
2. Verify PID exists.
3. Verify process executable is expected (`xray`/`v2ray`).
4. Verify command line references XRAT-owned config path.
5. If all checks pass, adopt monitoring ownership.
6. Otherwise mark prior session stale/failed with explicit reason and clear
   active config state.

Suggested pid path:

- `~/.local/state/xrat/xrat-xray.pid`

## Rotation Contract (Area #5 Bridge)

Phase 4.5 must expose make-before-break primitives so area #5 can layer rotation
without redesigning ownership.

Canonical handoff flow:

1. Running instance A is active.
2. Rotation event arrives (`manual`, `timer`, `health-check failed`).
3. Supervisor selects candidate B (via repository/rotation service).
4. Spawn B on alternate local inbound port(s).
5. Validate B readiness via internal proxy health check.
6. Atomically switch active routing/session ownership to B.
7. Gracefully stop A and persist cleanup transition.

If B fails validation:

- stop B
- persist failure reason for candidate B
- keep A active

## Suggested Delivery Order

1. Add `xrat daemon` command and supervisor bootstrap.
2. Add local IPC server and CLI client wiring for connect/disconnect/status.
3. Add supervisor event bus (`tokio::sync::mpsc`) and timer-driven health task.
4. Add restart reconciliation + strict reattach verification.
5. Add make-before-break replace primitive and transition reason taxonomy.
6. Add focused tests for daemon ownership, reattach mismatch, and safe handoff.

## Exit Criteria

- daemon updates runtime failure state without waiting for next CLI command
- CLI runtime commands operate through daemon IPC when daemon is running
- restart reconciliation and reattach policy are implemented and documented
- supervisor exposes safe replace/stop contracts for area #5 rotation work
- transition reasons are persisted and usable by future scheduler logic

## IPC Contract (Initial Shape)

Keep Phase 4 command semantics, but execute them via daemon RPC when available.
If daemon is not reachable, return explicit guidance (`xrat daemon start`) and
do not silently fall back to direct runtime ownership.

Suggested request types:

- `RuntimeConnect { selected_only: bool, include_geoip: bool }`
- `RuntimeDisconnect { reason: DisconnectReason }`
- `RuntimeStatus`
- `RuntimeReplace { trigger: RotationTrigger, candidate_id: Option<i64> }`
- `DaemonPing`

Suggested response envelope:

- `ok: bool`
- `code: "ok" | "busy" | "not_found" | "invalid_state" | "internal_error"`
- `message: String`
- `payload: Option<T>`

Use a single serialized schema (JSON or bincode) with version tag
(`protocol_version`) so later phases can evolve fields safely.

## Transition Reason Taxonomy

Persist reason codes as stable machine values and optional details for humans.
Prefer additive enums so downstream scheduler logic can depend on them.

Minimum reason code set:

- `manual_connect`
- `manual_disconnect`
- `daemon_restart_reattach_ok`
- `daemon_restart_reattach_rejected_pid_missing`
- `daemon_restart_reattach_rejected_exec_mismatch`
- `daemon_restart_reattach_rejected_cmdline_mismatch`
- `process_exit_unexpected`
- `health_check_failed`
- `replace_started`
- `replace_validation_failed`
- `replace_commit_success`
- `replace_rollback_keep_old`

Store:

- `reason_code` (required, stable)
- `reason_detail` (optional text for diagnostics)
- `origin` (`cli` | `daemon` | `health_task` | `rotation_task`)

## Data and Schema Additions

Phase 4.5 should avoid broad schema churn; add only fields required for
supervisor ownership and rotation bridge.

Likely additions:

- runtime session table:
  - `owner_kind` (`direct_cli` in Phase 4, `daemon` in Phase 4.5)
  - `owner_instance_id` (daemon boot UUID for traceability)
  - `last_transition_reason_code`
  - `last_transition_reason_detail`
- candidate/proxy state table (if not already present):
  - `cooldown_until`
  - `last_failed_at`
  - `last_failed_reason_code`

Migration rule: new columns must be nullable or have backward-safe defaults so
existing Phase 4 databases upgrade without manual repair.

## Implementation Checklist (Code-Level)

1. `src/cli/daemon.rs`
   - add `xrat daemon start|status|stop` command surface
   - add daemon socket path/port override flags (advanced/debug only)
2. `src/app/daemon/server.rs`
   - bind local transport, decode request envelope, route to supervisor bus
   - enforce single-flight timeout and structured error mapping
3. `src/app/daemon/supervisor.rs`
   - own runtime child process handle and watch task
   - process event queue (`connect`, `disconnect`, `replace`, `status`)
   - persist macro transition records with reason taxonomy
4. `src/app/commands/runtime/*`
   - refactor existing Phase 4 logic into reusable service functions callable by
     supervisor handlers
5. `src/app/runtime_service/*`
   - add reattach verifier (`pid`, executable, cmdline config path checks)
   - add handoff helper for make-before-break replace

## Test Matrix

Minimum targeted tests for Phase 4.5:

- CLI/IPC:
  - `runtime_status_returns_daemon_unreachable_hint`
  - `runtime_connect_routes_to_daemon_when_available`
- Reattach:
  - `reattach_accepts_matching_pid_exec_cmdline`
  - `reattach_rejects_pid_missing_marks_session_stale`
  - `reattach_rejects_exec_mismatch_marks_failed`
- Supervisor lifecycle:
  - `unexpected_process_exit_persists_failure_reason`
  - `disconnect_stops_active_runtime_and_clears_owner`
- Replace safety:
  - `replace_success_commits_new_runtime_then_stops_old`
  - `replace_validation_failure_keeps_old_runtime_active`

Prefer deterministic tests with fake runtime adapter traits rather than spawning
real Xray binaries in unit-level coverage.

## Next Work Slices (Implementation Order)

Use small, reviewable batches that each tighten one daemon contract.

### Slice A - Daemon command correctness hardening

Target files:

- `src/app/commands/daemon.rs`
- `src/app/daemon/client.rs`
- `src/app/daemon/protocol.rs`

Tasks:

- make `xrat daemon start` detach (or clearly spawn+return) so command does not
  hold foreground shell
- treat daemon replies with `ok=false` as command failure in `daemon status` and
  `daemon stop`
- enforce protocol compatibility gate and return a structured version error

Done when:

- start command returns promptly after daemon launch
- `daemon status` exits non-zero when daemon reports structured failure
- incompatible `protocol_version` request is rejected with explicit `code`

### Slice B - Supervisor error semantics and failure visibility

Target files:

- `src/app/daemon/supervisor.rs`
- `src/app/daemon/server.rs`
- `src/app/runtime_service/`

Tasks:

- stop masking runtime backend errors as `"unknown"` status
- map `RuntimeService::status()` failures to `ok=false` envelopes with stable
  error code
- persist unexpected runtime exit with `process_exit_unexpected` transition
  reason

Done when:

- status response differentiates `running`, `not_running`, and `backend_error`
- failure path includes reason code and detail text suitable for CLI display
- regression test asserts failure is visible through daemon IPC

### Slice C - Reattach verification and restart reconciliation

Target files:

- `src/app/runtime_service/reattach*.rs` (or equivalent module)
- `src/app/daemon/supervisor.rs`
- `src/db/repository/`

Tasks:

- add strict reattach verifier for `pid` existence, executable identity, and
  XRAT-owned config path in cmdline
- on reject, persist explicit `daemon_restart_reattach_rejected_*` reason and
  clear active owner state
- on accept, mark session with `owner_kind=daemon` and `owner_instance_id`

Done when:

- daemon restart deterministically chooses adopt vs reject path
- stale session metadata is cleaned up without requiring manual `disconnect`
- reattach tests cover all reject variants and acceptance case

### Slice D - Replace primitive for Area #5 bridge

Target files:

- `src/app/daemon/supervisor.rs`
- `src/app/runtime_service/`
- `src/app/daemon/protocol.rs`

Tasks:

- add `RuntimeReplace` request handling with make-before-break flow
- validate candidate readiness before switching active ownership
- on validation failure, keep old runtime active and persist rollback reason

Done when:

- successful replace commits new runtime then drains/stops old runtime
- failed replace preserves old runtime availability
- replace responses include reason-coded outcomes for scheduler consumption

Current status update (2026-05-11):

- runtime service replace now stages candidate first (alternate ports), then
  flips active config and stops old session
- rollback behavior on validation/spawn failure keeps old runtime active and
  persists rollback reason metadata
- remaining: add deterministic success-path test asserting new-runtime-commit
  then old-runtime-stop ordering, and broaden daemon-level integration coverage

## Test Closure Checklist

Before Phase 4.5 is marked complete, verify all of the following in CI:

- daemon command behavior:
  - detached start contract validated
  - `status` and `stop` fail correctly on `ok=false` responses
- protocol behavior:
  - incompatible protocol version is rejected
- supervisor behavior:
  - runtime status backend failures surface as structured daemon errors
  - unexpected process exit is persisted with stable reason code
- reattach behavior:
  - accept path and each reject path (`pid_missing`, `exec_mismatch`,
    `cmdline_mismatch`) have deterministic regression tests
- replace behavior:
  - success handoff and rollback keep-old-runtime scenario are both covered

## Definition of Done (Phase 4.5)

Phase 4.5 is complete only when all of these are true simultaneously:

- daemon is the only runtime owner while it is running (no silent CLI fallback)
- runtime failure is detectable and persisted without user-triggered commands
- restart reconciliation follows strict reattach policy with explicit reasons
- replace contract is available and safe for Area #5 rotation integration
- schema/repository changes for owner + transition reason fields are migrated
  and backward-safe
