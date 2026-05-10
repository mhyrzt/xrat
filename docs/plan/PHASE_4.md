# Phase 4 Managed Xray Runtime

## Goal

Build the long-lived Xray runtime layer for XRAT on top of the temporary
probe/runtime foundation introduced in Phase 3.

By the end of this phase, XRAT should be able to:

- choose one stored config as the runtime target
- generate a runnable Xray config for sustained local use
- launch Xray as a managed child process from Rust
- expose runtime-oriented CLI commands such as:
  - `xrat connect <id>`
  - `xrat disconnect`
  - `xrat status`
- track the active session and persist runtime state in the configured database
  backend
- stop, restart, or switch configs cleanly without leaving orphaned processes
  behind

This phase is about turning Xray from a disposable probe tool into a managed
application runtime.

## Validation Link

Related parity checklist source:

- `docs/validation/0_xray-knife_vs_xrat_gap_checklist.md`
  - section **4) Engine Selection Logic (xray vs sing-box)**
  - section **5) Auto-Rotating Proxy**

When those checklist items change status, update this phase doc assumptions for
runtime engine selection and process ownership semantics.

## Why This Phase Exists

Phase 3 gives XRAT a way to validate configs and measure them with real traffic.

That is enough to answer:

- does this config work?
- how fast is it right now?

But it still does not give the user a real usable proxy session.

Phase 4 exists to answer the next operational question:

- can XRAT take one stored, tested config and actually run it as the user's
  active Xray instance?

This phase turns the project from a config database plus tester into a client
that can drive a real local proxy runtime.

## Current Starting Point

Phase 4 should explicitly build on the runtime primitives and data-model work
completed through Phase 3.6.

Those reusable pieces include:

- converting a stored node into runnable Xray JSON through `src/xray/config/`
- generating probe configs and initial runtime configs with SOCKS plus optional
  HTTP inbounds
- spawning Xray as a child process through `src/xray/process.rs`
- writing temporary config files and waiting for local port readiness
- collecting startup stderr when the process exits early
- stopping a child process on explicit kill or drop
- storing runtime session rows through the database facade and repository layer
- tracking config `selected`, `active`, and `enabled` flags
- using concrete app/DB error types instead of boxed errors
- using tracing for diagnostics instead of ad-hoc stderr prints
- dispatching DB behavior across SQLite and PostgreSQL
- verifying PostgreSQL behavior against a real Docker Compose database
- preserving stable node identity with canonical `configs.dedup_key` values

Phase 3 uses those pieces for short-lived tests.

Phase 4 reuses them for long-lived user sessions.

The boundary is:

- Phase 3 = disposable probe runtime
- Phase 4 = managed persistent runtime

## Scope Boundary

Phase 4 should cover:

- runtime config generation for one chosen config
- managed Xray process lifecycle
- runtime session persistence in `runtime_sessions`
- tracking active config, local ports, PID, and status
- CLI commands for connect/disconnect/status
- switching from one running config to another safely
- clear user-visible runtime errors
- SQLite and PostgreSQL support for the runtime repository path

Phase 4 should not yet cover:

- daemon IPC runtime ownership (`xrat daemon`)
- continuous background crash monitoring after CLI exit
- make-before-break auto-rotation orchestration
- the HTTP API
- the TUI runtime dashboard
- scheduled health checks or auto-reconnect policies
- multi-profile balancing or load-sharing
- advanced server orchestration beyond one active runtime session

Those belong to later phases.

## Desired User Experience

The first usable version should feel like this:

- `xrat connect 12`
  - loads config `12`
  - generates a runnable Xray config
  - starts Xray locally
  - marks the session as running
  - prints the local proxy ports and current status

- `xrat status`
  - shows whether Xray is running
  - shows the active config id
  - shows local ports
  - shows process id if available
  - shows when the session started

- `xrat disconnect`
  - stops the managed Xray process
  - updates runtime state in the configured database
  - confirms the runtime is no longer active

Later extensions in or after this phase can add:

- `xrat reconnect`
- `xrat connect selected`
- `xrat logs`
- `xrat restart`

But the minimum useful slice is `connect`, `disconnect`, and `status`.

## Runtime Model

Phase 4 should assume one primary active runtime session at a time.

That keeps the first implementation simple and matches the normal user
expectation for a desktop/local proxy tool.

Recommended runtime concepts:

- selected config
  - the config the user prefers
- active config
  - the config currently running in Xray
- runtime session
  - the process lifecycle record for the current or most recent run

A config can be tested many times, but only one config should normally be active
at once.

## Local Runtime Shape

The managed runtime should use a full local proxy configuration rather than the
tiny probe configuration from Phase 3.

The codebase already has:

- `[runtime.socks]`, `[runtime.http]`, `[runtime.shadowsocks]`, and
  `[runtime.sniffing]` config sections
- `generate_runtime_config(node, socks_port, http_port)` for SOCKS plus optional
  HTTP inbounds
- runtime defaults such as `engine`, `replace_active_session`, and inbound host
  and port values

Recommended initial runtime behavior:

- bind local SOCKS and/or HTTP proxy ports using `[runtime.*]` settings
- generate a stable runtime config for the chosen node
- start Xray with that config
- keep the child process alive until the user disconnects or it exits
  unexpectedly
- validate that at least one local inbound is enabled before startup
- honor the configured runtime engine path/name; initially this can still map to
  the `xray` executable

Possible first runtime modes:

- SOCKS only
- HTTP only
- Shadowsocks only
- both SOCKS and HTTP
- any enabled combination of SOCKS, HTTP, and Shadowsocks
- mixed inbound if that matches the intended Xray setup

A good first product choice is:

- start with configured SOCKS, HTTP, and Shadowsocks inbounds
- avoid global/TUN mode in this phase

That keeps platform-specific complexity out of the initial runtime work.

## Persistence Model

The project already has a `runtime_sessions` table.

Phase 4 should make it the source of truth for runtime history and latest known
state.

The table should capture at least:

- config id
- status
- local port information
- process id if known
- XRAT-owned failure reason if known
- started time
- stopped time
- updated time

Current schema note:

- Phase 4 uses explicit nullable host/port columns for SOCKS, HTTP, and
  Shadowsocks inbounds so persisted status reflects the launched session shape.
- The earlier `mixed_port` column is removed by migration and should not be used
  by runtime code.
- Phase 4 stores a short `failure_reason` owned by XRAT for lifecycle failures
  such as startup failure or stale PID reconciliation. Xray/V2Ray still owns the
  detailed runtime log files; the database reason should be a concise summary,
  not a copy of external logs.
- If arbitrary custom inbounds are added later, a separate JSON inbound summary
  can be added without overloading the fixed inbound columns.

Recommended session states:

- `starting`
- `running`
- `stopping`
- `stopped`
- `failed`

Expected flow:

1. insert or update session as `starting`
2. spawn Xray and wait for readiness
3. mark session `running`
4. on user disconnect, mark `stopping`, terminate process, then mark `stopped`
5. on unexpected exit, mark `failed`

## Relationship To Config State

Phase 4 should integrate with the existing config flags in `configs`.

Recommended behavior:

- `connect <id>` should set that config as active if startup succeeds
- if another config is active, it should be deactivated during a successful
  switch
- disabled or deleted configs should not be connectable unless the product
  explicitly adds an override later
- if the config record is deleted after a runtime has started, the existing
  Xray/V2Ray process should keep running from the generated runtime config file;
  `status` should continue to show the persisted session and label the session
  config as missing/deleted instead of rereading mutable config state
- future delete/disable commands should either refuse to remove the active
  config until `disconnect` succeeds or require an explicit force path that also
  stops the runtime
- `status` should read both runtime session state and active config state
  cleanly

This keeps runtime state and config lifecycle state aligned.

## Recommended Module Layout

Phase 4 should build on the structure started in Phase 3.

Suggested modules:

- `src/xray/mod.rs`
- `src/xray/config.rs`
- `src/xray/process.rs`
- `src/xray/runtime.rs`
- `src/app/commands/connect.rs`
- `src/app/commands/disconnect.rs`
- `src/app/commands/status.rs`
- `src/cli/connect.rs`
- `src/cli/disconnect.rs`
- `src/cli/status.rs`

Responsibilities:

- `src/xray/config.rs`
  - build full runnable Xray configs for managed sessions
- `src/xray/process.rs`
  - process spawning, readiness waiting, shutdown, and output capture
- `src/xray/runtime.rs`
  - higher-level lifecycle orchestration and DB updates
- CLI command files
  - parse args and call runtime service functions

## Detailed Implementation Plan

### Step 1. Stabilize reusable Xray runtime primitives - Done

Take the process/config helpers introduced for Phase 3 and make sure they can
support long-lived sessions.

That includes:

- stable config generation APIs that accept runtime settings, not just raw ports
- explicit local port selection and validation
- support for configured bind hosts where safe
- readiness detection
- structured shutdown handling
- stderr/stdout capture for diagnostics
- keeping generated runtime config files in a predictable runtime directory when
  useful for debugging, instead of always relying on anonymous temp files

This step should reduce duplication between probe mode and managed mode.

Important refactor:

- keep probe-specific behavior available for testing
- add managed-runtime-specific generation/launch paths rather than stretching
  probe assumptions into long-lived sessions
- make `XrayProcess` readiness checks work with the primary inbound selected for
  the managed runtime

### Step 2. Define runtime session service types - Done

Add domain types for:

- runtime launch request
- runtime launch result
- runtime status snapshot
- shutdown result
- process failure details

These types should sit above raw DB rows so CLI and future TUI/API code can
consume a stable runtime model.

They should also translate lower-level errors into the existing concrete
`AppError`/`DbError` style rather than introducing boxed errors.

### Step 3. Implement `connect` - Done

Add `xrat connect <id>`.

Recommended behavior:

- load the config by id
- reject missing or disabled configs
- if another session is already running, either:
  - stop it first when `[runtime].replace_active_session = true`, or
  - fail with a clear message when replacement is disabled
- generate a managed runtime config
- insert/update `runtime_sessions` as `starting`
- spawn Xray and wait for readiness
- mark session `running`
- mark the config active
- preserve the selected flag independently from active state
- print local port details

A later iteration can support `connect selected`, but id-based startup is enough
first.

### Step 4. Implement `disconnect` - Done

Add `xrat disconnect`.

Recommended behavior:

- load the current running session
- mark it `stopping`
- terminate the child process cleanly using the saved PID/process handle path
- wait for exit with a bounded timeout
- force-kill only if needed
- mark session `stopped`
- clear active runtime state in config/session tables as appropriate
- handle stale PIDs by marking the session stopped or failed with a clear reason

This command should be safe to run even if nothing is active; it should just
report that nothing is running.

### Step 5. Implement `status` - Done

Add `xrat status`.

Recommended output:

- current runtime status
- active config id/name
- local ports
- process id
- started time
- last failure if the most recent session failed
- selected config, if different from the active config
- database backend label if useful for debugging

This command should use persisted state plus lightweight runtime checks where
appropriate.

### Step 6. Handle switching and unexpected exits - Mostly done

After basic connect/disconnect/status works, harden runtime lifecycle behavior.

Important cases:

- connecting a new config while another one is running
- Xray exits unexpectedly after a successful start
- the saved PID is stale
- the process starts but never becomes ready
- shutdown hangs and needs escalation to force-kill
- config flags and runtime session rows drift out of sync
- the app restarts while the previously launched process may still be running

The runtime service should classify these clearly and keep the DB state honest.

### Step 7. Add focused tests - Partially done

Add coverage for:

- CLI parsing for `connect`, `disconnect`, and `status`
- runtime session repository updates
- SQLite and PostgreSQL runtime-session behavior where practical
- transition rules between `starting`, `running`, `stopping`, `stopped`, and
  `failed`
- connect rejection for deleted/disabled configs
- unexpected process exit handling through a fake process adapter where possible
- runtime config generation from `[runtime.socks]` and `[runtime.http]` settings
- status behavior for stale PID/session state

As in Phase 3, keep the subprocess boundary small so most logic is testable
without a real Xray binary.

## Process Management Strategy

A strong implementation detail for this phase is to avoid scattering
child-process ownership across the app.

Recommended rule:

- one runtime service should be responsible for spawning, tracking, and shutting
  down Xray

That service should:

- record PID when available
- retain enough state to stop the right process
- persist state transitions immediately
- capture startup or crash errors in a way `status` and future UI can surface
- emit diagnostics through `tracing`

If the app is restarted while Xray is still running, Phase 4 should at least
detect and report the mismatch, even if full re-attachment is deferred.

For the first implementation, avoid relying on in-memory child ownership alone.
The CLI process may exit after `connect`, so `disconnect` and `status` need to
work from persisted session data plus OS-level process checks.

Continuous monitoring after the CLI exits is intentionally deferred to Phase
4.5. Phase 4 reconciles runtime exits on the next `status`, `connect`, or
`disconnect` command instead of running a background daemon/watcher.

## Phase 4 to 4.5 Handoff Contract

Phase 4 should leave stable seams so Phase 4.5 can move ownership to an explicit
daemon without rewriting command semantics.

Recommended handoff rules:

- keep connect/disconnect/status behavior defined as service operations instead
  of tightly coupling them to direct child-process ownership in CLI handlers
- persist transition reasons in `runtime_sessions.failure_reason` using concise,
  XRAT-owned reason labels
- keep session writes deterministic (`starting` -> `running` -> `stopping` ->
  `stopped` or `failed`) so daemon reconciliation can reuse the same state
  machine
- treat stale PID reconciliation as a first-class runtime outcome with explicit
  status updates
- avoid one-off runtime side channels that bypass repository/service layers

Expected Phase 4.5 migration path:

- Phase 4 command handlers call local runtime services directly
- Phase 4.5 command handlers become IPC clients to `xrat daemon`
- runtime service logic remains mostly unchanged and is hosted by the daemon

## Suggested Command Semantics

### `xrat connect <id>`

Purpose:

- run one stored config as the active local proxy session

Implemented flags:

- `--json`

Possible future flags:

- `--socks-port <port>`
- `--http-port <port>`
- `--replace`
- `--background`

Initial behavior reads default ports and replacement policy from `[runtime]`.

Note for Phase 4.5 alignment:

- `--background` should only become active when daemon ownership is introduced;
  in Phase 4 it remains non-goal behavior.

### `xrat disconnect`

Purpose:

- stop the current managed runtime session

Implemented flags:

- `--json`

Possible future flags:

- `--force`

### `xrat status`

Purpose:

- show current runtime state and active config at a glance

Implemented flags:

- `--json`

## Recommended Delivery Order

To keep risk low, build this phase in the following order:

1. migrate `runtime_sessions` to explicit inbound port columns and remove
   `mixed_port`
2. stabilize shared Xray config/process helpers around `[runtime]` settings
3. define runtime session domain/service types
4. ship `connect <id>` end to end
5. ship `disconnect`
6. ship `status`
7. harden switching, stale PID, and crash handling
8. add focused unit/repository/CLI/runtime tests

## Implementation Progress

Estimated completion: 90%.

Initial Phase 4 work has started with the minimum managed runtime command
surface:

- added `xrat connect <id>`, `xrat disconnect`, and `xrat status` CLI parsing
- added detached Xray startup helpers that write per-session runtime config and
  log files under the XRAT runtime directory
- connected runtime startup to `[runtime].engine`, `[runtime.socks]`, and
  `[runtime.http]`
- persisted `starting`, `running`, `stopping`, `stopped`, and `failed` session
  transitions through the existing `runtime_sessions` repository path
- wired successful connects to `configs.is_active` while preserving
  `configs.is_selected`
- added PID-based status checks and stale-session reporting for the first status
  slice
- moved shared runtime lifecycle checks behind a reusable `RuntimeService`
- hardened disconnect with graceful terminate, bounded wait, and force-kill
  fallback
- reconciled stale `starting`, `running`, and `stopping` sessions before
  connect/status so active config state does not stay stuck on dead processes
- added explicit nullable runtime session columns for SOCKS, HTTP, and
  Shadowsocks inbound host/port values and removed the old `mixed_port` column
- persisted the launched runtime inbound shape in `runtime_sessions` so `status`
  reports the session that was actually started instead of rereading mutable
  current config defaults
- included SOCKS, HTTP, and Shadowsocks runtime inbounds in managed runtime
  generation and status output
- removed the obsolete mixed inbound path from managed runtime planning and code
- added host-machine address display when an inbound binds to `0.0.0.0`
- extracted managed runtime orchestration into `RuntimeService` with reusable
  connect, disconnect, status, endpoint, and status snapshot types for future
  TUI/API use
- added local inbound liveness checks to `status`, including per-inbound
  reachable/unreachable output and a `degraded` runtime status when the process
  is alive but an expected local listener is closed
- added a concise `runtime_sessions.failure_reason` field for XRAT-owned
  lifecycle summaries while leaving detailed Xray/V2Ray logs in their generated
  log files
- added `--json` output for `connect`, `disconnect`, and `status` so future
  TUI/API/script callers can consume structured runtime results
- documented that deleting a config after startup does not invalidate the
  already-generated runtime config; status should report the session config as
  missing/deleted when the DB row is gone
- deferred continuous daemon/background watcher behavior to Phase 4.5

Remaining Phase 4 work:

- add fake-Xray lifecycle tests for connect, disconnect, replacement, stale PID,
  and startup failure behavior

## Completion Criteria

Phase 4 can be considered complete when:

1. [x] XRAT exposes `xrat connect <id>`, `xrat disconnect`, and `xrat status`
2. [x] one stored config can be launched as a managed local Xray runtime
3. [x] runtime session state is persisted in `runtime_sessions` for SQLite and
       PostgreSQL
4. [x] active config and runtime state stay aligned during normal start/stop
       flows
5. [x] switching or shutdown does not leave orphaned Xray processes behind
6. [x] unexpected startup and runtime failures are surfaced clearly enough for
       users to diagnose issues
7. [ ] the new code is covered by focused CLI, repository, and lifecycle tests
8. [x] generated runtime config respects the relevant `[runtime]` settings

## Open Questions

These should be resolved while implementing, but they should not block starting
the phase:

- which local ports should be fixed by default versus allocated dynamically?
