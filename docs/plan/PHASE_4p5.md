# Phase 4.5 Runtime Supervisor, Daemon IPC, and Reattach Policy

## Goal

Introduce an explicit long-lived XRAT daemon that owns runtime process
lifecycle, watches failures in real time, and provides stable ownership
contracts for area #5 (auto-rotating proxy).

Phase 4 keeps XRAT command-driven (`connect`/`disconnect`/`status`) with
reconciliation on the next CLI call. Phase 4.5 moves runtime ownership to a
single background supervisor so crash detection, rotation triggers, and session
state updates do not depend on user-initiated commands.

## Progress Report (2026-05-10)

Current implementation has landed the first daemon/IPC slice, but Phase 4.5 is
not yet complete.

### Completed

- Added `xrat daemon start|status|stop` CLI surface and parser/help coverage.
- Added local Unix socket IPC server/client with request envelope and
  `protocol_version` field.
- Added supervisor event channel for `ping`, `status`, `connect`,
  `disconnect`, and `shutdown`.
- Wired `connect`/`disconnect`/`status` commands to prefer daemon IPC when
  reachable.
- Added daemon server tests for startup conflict and shutdown behavior.

### In Progress / Partial

- Runtime supervision exists, but current supervisor status handling drops
  runtime backend errors and reports `"unknown"` status instead of surfacing the
  failure.
- IPC response envelope supports `ok/code/message/payload`, but daemon command
  handlers do not consistently treat `ok=false` as command failure.
- Protocol field exists in request envelope, but server routing does not yet
  enforce compatibility checks.

### Not Started (Phase 4.5 scope items)

- True daemonization/detach behavior for `xrat daemon start` (current command
  blocks foreground).
- Reattach verification policy (`pid`/executable/cmdline checks) and restart
  reconciliation.
- Make-before-break replace primitive (`RuntimeReplace`) and rotation trigger
  flow.
- Transition reason taxonomy persistence and schema additions (`owner_kind`,
  `owner_instance_id`, transition reason fields, cooldown/failure tracking).
- Timer-driven health task and rotation-oriented failure signaling.

### Review Findings Snapshot (current diff)

These findings came from reviewing the current daemon-related changes and should
be resolved before marking Phase 4.5 complete:

- High: `xrat daemon start` is not daemonized and blocks terminal execution.
  - `src/app/commands/daemon.rs:10`
  - `src/app/commands/daemon.rs:15`
- Medium: daemon `status`/`stop` command paths ignore `response.ok` and can
  still return success when daemon reports failure.
  - `src/app/commands/daemon.rs:18`
  - `src/app/commands/daemon.rs:45`
- Medium: supervisor runtime status path masks `RuntimeService::status()`
  errors as `"unknown"` with successful envelope semantics.
  - `src/app/daemon/supervisor.rs:75`
- Low: server accepts any `protocol_version`; no compatibility gate yet.
  - `src/app/daemon/server.rs:339`

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

Prefer deterministic tests with fake runtime adapter traits rather than
spawning real Xray binaries in unit-level coverage.
