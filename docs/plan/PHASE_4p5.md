# Phase 4.5 Runtime Supervisor, Daemon IPC, and Reattach Policy

## Goal

Introduce an explicit long-lived XRAT daemon that owns runtime process
lifecycle, watches failures in real time, and provides stable ownership
contracts for area #5 (auto-rotating proxy).

Phase 4 keeps XRAT command-driven (`connect`/`disconnect`/`status`) with
reconciliation on the next CLI call. Phase 4.5 moves runtime ownership to a
single background supervisor so crash detection, rotation triggers, and session
state updates do not depend on user-initiated commands.

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
