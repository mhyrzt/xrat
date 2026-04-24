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
- track the active session and persist runtime state in SQLite
- stop, restart, or switch configs cleanly without leaving orphaned processes
  behind

This phase is about turning Xray from a disposable probe tool into a managed
application runtime.

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

## Dependency On Phase 3

Phase 4 should explicitly build on the runtime primitives pulled into Phase 3.

Those reusable pieces include:

- converting a stored node into runnable Xray JSON
- choosing local ports safely
- spawning Xray as a child process
- waiting for readiness
- collecting process output and failure details
- stopping a child process reliably

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

Phase 4 should not yet cover:

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
  - updates runtime state in SQLite
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

Recommended initial runtime behavior:

- bind local SOCKS and/or HTTP proxy ports on `127.0.0.1`
- generate a stable runtime config for the chosen node
- start Xray with that config
- keep the child process alive until the user disconnects or it exits
  unexpectedly

Possible first runtime modes:

- SOCKS only
- HTTP only
- both SOCKS and HTTP
- mixed inbound if that matches the intended Xray setup

A good first product choice is:

- start with SOCKS plus HTTP or one mixed inbound
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
- started time
- stopped time
- updated time

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

### Step 1. Stabilize reusable Xray runtime primitives

Take the temporary process/config helpers introduced for Phase 3 and make sure
they can support long-lived sessions.

That includes:

- stable config generation APIs
- explicit local port selection
- readiness detection
- structured shutdown handling
- stderr/stdout capture for diagnostics

This step should reduce duplication between probe mode and managed mode.

### Step 2. Define runtime session service types

Add domain types for:

- runtime launch request
- runtime launch result
- runtime status snapshot
- shutdown result
- process failure details

These types should sit above raw DB rows so CLI and future TUI/API code can
consume a stable runtime model.

### Step 3. Implement `connect`

Add `xrat connect <id>`.

Recommended behavior:

- load the config by id
- reject deleted or disabled configs
- if another session is already running, either:
  - stop it first, then continue, or
  - fail with a clear message in the first iteration
- generate a managed runtime config
- insert/update `runtime_sessions` as `starting`
- spawn Xray and wait for readiness
- mark session `running`
- mark the config active
- print local port details

A later iteration can support `connect selected`, but id-based startup is enough
first.

### Step 4. Implement `disconnect`

Add `xrat disconnect`.

Recommended behavior:

- load the current running session
- mark it `stopping`
- terminate the child process cleanly
- wait for exit with a bounded timeout
- force-kill only if needed
- mark session `stopped`
- clear active runtime state in config/session tables as appropriate

This command should be safe to run even if nothing is active; it should just
report that nothing is running.

### Step 5. Implement `status`

Add `xrat status`.

Recommended output:

- current runtime status
- active config id/name
- local ports
- process id
- started time
- last failure if the most recent session failed

This command should use persisted state plus lightweight runtime checks where
appropriate.

### Step 6. Handle switching and unexpected exits

After basic connect/disconnect/status works, harden runtime lifecycle behavior.

Important cases:

- connecting a new config while another one is running
- Xray exits unexpectedly after a successful start
- the saved PID is stale
- the process starts but never becomes ready
- shutdown hangs and needs escalation to force-kill

The runtime service should classify these clearly and keep the DB state honest.

### Step 7. Add focused tests

Add coverage for:

- CLI parsing for `connect`, `disconnect`, and `status`
- runtime session repository updates
- transition rules between `starting`, `running`, `stopping`, `stopped`, and
  `failed`
- connect rejection for deleted/disabled configs
- unexpected process exit handling through a fake process adapter where possible

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

If the app is restarted while Xray is still running, Phase 4 should at least
detect and report the mismatch, even if full re-attachment is deferred.

## Suggested Command Semantics

### `xrat connect <id>`

Purpose:

- run one stored config as the active local proxy session

Possible future flags:

- `--socks-port <port>`
- `--http-port <port>`
- `--mixed-port <port>`
- `--replace`
- `--background`

The first iteration does not need all of these.

### `xrat disconnect`

Purpose:

- stop the current managed runtime session

Possible future flags:

- `--force`

### `xrat status`

Purpose:

- show current runtime state and active config at a glance

Possible future flags:

- `--json`

## Recommended Delivery Order

To keep risk low, build this phase in the following order:

1. stabilize shared Xray config/process helpers
2. define runtime session domain types
3. ship `connect <id>` end to end
4. ship `disconnect`
5. ship `status`
6. harden switching and crash handling
7. add focused unit/repository/CLI/runtime tests

## Completion Criteria

Phase 4 can be considered complete when:

1. XRAT exposes `xrat connect <id>`, `xrat disconnect`, and `xrat status`
2. one stored config can be launched as a managed local Xray runtime
3. runtime session state is persisted in `runtime_sessions`
4. active config and runtime state stay aligned during normal start/stop flows
5. switching or shutdown does not leave orphaned Xray processes behind
6. unexpected startup and runtime failures are surfaced clearly enough for users
   to diagnose issues
7. the new code is covered by focused CLI, repository, and lifecycle tests

## Open Questions

These should be resolved while implementing, but they should not block starting
the phase:

- should the first runtime expose SOCKS, HTTP, mixed inbound, or more than one
  at once?
- should `connect <id>` automatically replace an existing running session or
  require explicit confirmation later?
- should the app attempt to reattach to an already-running Xray process after
  restart, or just report a stale session?
- should `status` be purely DB-driven, or also verify that the PID still exists?
- which local ports should be fixed by default versus allocated dynamically?
