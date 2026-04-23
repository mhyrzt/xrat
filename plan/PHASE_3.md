# Phase 3 Connection Testing

## Goal

Add a real connection-testing layer to XRAT so stored configs can be measured, compared, and surfaced as usable candidates for later runtime selection.

By the end of this phase, XRAT should be able to:

- run a lightweight TCP reachability check for one stored config
- run a more meaningful real-delay check using actual proxy traffic
- generate a temporary runnable Xray config for one node
- launch and clean up a short-lived Xray subprocess used only for probing
- save both the historical result and the latest summary in SQLite
- expose this behavior through a focused CLI command such as:
  - `xrat test <id>`

This phase is about validating stored configs. It does include a minimal disposable Xray runtime for probing, but it does not include full long-lived runtime management yet.

## Why This Phase Exists

Phase 2 established persistence and Phase 2.5 established a CLI shape.

The next missing capability is confidence.

Right now XRAT can store nodes, list them, and manage their lifecycle state, but it still cannot answer the most important operational questions:

- does this config connect at all?
- how fast is it right now?
- which saved config is actually worth using?
- what was the last known health of a config?

Without this phase, Phase 4 would be forced to guess which config should be run.

Connection testing gives the app a factual basis for later actions such as:

- choosing a candidate for `connect`
- sorting configs in the future TUI/API
- filtering out clearly dead nodes
- showing recent failure reasons to the user

## Scope Boundary

Phase 3 should cover:

- test execution for persisted configs
- TCP reachability timing
- real-delay timing through actual proxy traffic
- the smallest Xray runtime layer required to support real-delay tests
- persistence of test history in `connection_tests`
- fast access to the latest known result for one config
- a CLI entrypoint for triggering tests
- focused tests for the tester and persistence flow

Phase 3 should not yet cover:

- launching Xray as a managed long-running session
- connect/disconnect/status user flows for persistent runtime control
- a full TUI testing dashboard
- HTTP endpoints for test results
- automated scheduling or background health polling

Those belong to later phases.

## Current Starting Point

The codebase already has part of the persistence groundwork for this phase:

- `connection_tests` table exists in `migrations/0001_init.sql`
- `src/db/model/connection_tests.rs` already defines insert and record models
- `src/db/repository/connection_tests.rs` already supports:
  - insert
  - list history by config
  - get latest result by config
- the CLI structure already has room for a future `test` command from Phase 2.5

What is still missing is the actual testing runtime and the command flow that uses it.

It is also now clear that real-delay testing depends on a small subset of the original Phase 4 work. Because of that, this phase intentionally pulls in only the runtime pieces needed for disposable probe execution.

## Desired User Experience

The first usable version should feel like this:

- `xrat test 12`
  - loads config `12` from SQLite
  - performs a TCP connectivity check
  - performs a real-delay test if TCP succeeds
  - writes the result to `connection_tests`
  - prints a short human-readable summary

A later extension inside the same phase can add batch modes such as:

- `xrat test selected`
- `xrat test --enabled-only`
- `xrat test --subscription 3`

But the minimum useful slice is one-config testing by id.

## Testing Model

Phase 3 should treat connection testing as two related but distinct checks.

### 1. TCP reachability check

Purpose:

- answer whether the remote address and port are reachable at all
- measure raw socket connect time
- fail quickly on DNS, timeout, or connection-refused cases

This is the cheap test and should normally run first.

Expected output fields:

- `tcp_ok`
- `tcp_ms`
- `failure_kind`
- `failure_reason`

### 2. Real-delay check

Purpose:

- measure usable latency through an actual proxy path
- confirm more than just an open TCP port
- provide the number that later UX should use for sorting and quality decisions

This should not be based only on opening a socket.

It should involve real proxied traffic, for example:

- generate a temporary Xray config for one node
- start a short-lived local Xray process bound to an ephemeral local port
- send an HTTP request through that local proxy to a known target
- measure total request time
- tear the process down immediately after the measurement

Expected output fields:

- `real_delay_ok`
- `real_delay_ms`
- `failure_kind`
- `failure_reason`

## Real-Delay Strategy

The most practical path is to implement real-delay testing with a short-lived Xray subprocess instead of trying to reimplement protocol handshakes in Rust.

Recommended flow:

1. Load one stored config from SQLite.
2. Generate a temporary Xray runtime config for that single node.
3. Bind a local SOCKS or mixed inbound on an available ephemeral port.
4. Launch Xray as a child process.
5. Wait until the local inbound is accepting connections.
6. Send one HTTP request through the local proxy to a fixed test URL.
7. Measure total elapsed time.
8. Capture success or classify the failure.
9. Kill the subprocess and remove temporary files.

This gives Phase 3 a realistic measurement while still keeping Phase 4 focused on long-lived runtime lifecycle management later.

## Recommended Target URL Rules

The real-delay probe needs a deterministic target.

Recommended initial rules:

- start with one default target URL in config/code
- prefer a lightweight HTTPS endpoint with a small response body
- use a short timeout budget
- make the target configurable later if needed

A good initial product decision is:

- hardcode one default target for Phase 3
- avoid adding user-facing configuration until the tester is stable

That keeps this phase focused on correctness rather than configurability.

## Failure Classification

The current schema already supports:

- `dns`
- `timeout`
- `refused`
- `tls`
- `auth`
- `process`
- `unknown`

Phase 3 should consistently map tester failures into those buckets.

Recommended interpretation:

- `dns`: hostname resolution failure
- `timeout`: dial, handshake, readiness, or probe timeout
- `refused`: TCP refused by destination or local proxy bootstrap refusal
- `tls`: TLS handshake or certificate-layer failure during real-delay probe
- `auth`: proxy authentication failure where detectable
- `process`: Xray failed to start, crashed, or exited before probe completion
- `unknown`: anything else not classified cleanly

The free-form `failure_reason` should keep the short actionable message.

## Data Model Recommendation

The project already stores full test history in `connection_tests`.

That is good and should remain the source of truth.

However, the roadmap also says the latest result should be cached for fast UI display. The cleanest way to reach that is to add a latest-result summary close to `configs` rather than forcing every future UI/API path to re-scan history.

Recommended addition in this phase:

- add a migration that stores latest test summary fields on `configs`, for example:
  - `last_tcp_ok`
  - `last_tcp_ms`
  - `last_real_delay_ok`
  - `last_real_delay_ms`
  - `last_failure_kind`
  - `last_failure_reason`
  - `last_tested_at`

Alternative:

- keep history only and read latest via `ORDER BY tested_at DESC`

That alternative is acceptable for a first pass, but the cached summary is more aligned with the roadmap and future TUI/API needs.

## Recommended Module Layout

Add a new testing area rather than mixing this logic into `app` or `db`.

Suggested structure:

- `src/tester/mod.rs`
- `src/tester/model.rs`
- `src/tester/tcp.rs`
- `src/tester/real_delay.rs`
- `src/tester/failure.rs`
- `src/tester/service.rs`
- `src/xray/mod.rs`
- `src/xray/config.rs`
- `src/xray/process.rs`

Responsibilities:

- `model.rs`
  - request/result structs for the testing layer
- `tcp.rs`
  - TCP reachability timing
- `real_delay.rs`
  - short-lived Xray probe and proxied request measurement
- `failure.rs`
  - failure classification helpers
- `service.rs`
  - orchestration of config loading, running both checks, and persisting results
- `src/xray/config.rs`
  - convert one stored node into a minimal runnable Xray config for probing
- `src/xray/process.rs`
  - spawn Xray, wait for readiness, and shut it down cleanly

CLI wiring should then remain thin:

- `src/cli/test.rs`
- `src/app/commands/test.rs`

## Relationship To Phase 4

Phase 3 intentionally pulls a thin runtime sub-layer forward because real-delay testing depends on it.

The key boundary is:

- Phase 3 uses Xray as a disposable probe runtime
- Phase 4 uses Xray as a managed long-lived application runtime

So this phase should implement only the reusable runtime primitives needed for testing:

- generate a temporary Xray config
- pick a local port
- spawn Xray
- wait for readiness
- stop the process and clean up

Phase 4 should then build the full runtime UX on top of those primitives:

- `connect`
- `disconnect`
- `status`
- selected/active session management
- restart/switch logic

So this phase should build the smallest reusable runtime slice necessary for testing, not the entire runtime system.

## Detailed Implementation Plan

### Step 1. Add the CLI command surface

Add a new command in the existing command-first CLI:

- `xrat test <id>`

Initial behavior:

- accept one config id
- optionally allow a `--tcp-only` mode for easier incremental development
- print a concise summary of:
  - config id
  - tcp status and timing
  - real-delay status and timing
  - failure reason when relevant

This step should only add argument parsing and command dispatch once the tester service shape is known.

### Step 2. Introduce tester domain types

Add explicit types for:

- test request
- tcp result
- real-delay result
- combined persisted result
- failure classification

This avoids overloading the DB insert model with runtime-only state and makes testing easier.

Example concerns these types should capture:

- timeout settings
- chosen test mode
- target URL
- local probe port
- elapsed durations before conversion to DB fields

### Step 3. Implement TCP testing

Build a standalone async TCP tester using `tokio::net::TcpStream` plus timeout control.

Behavior:

- resolve the config address
- attempt connect to `address:port`
- measure elapsed milliseconds
- classify errors into the failure buckets

This is the first deliverable because:

- it has no Xray dependency
- it proves config lookup and persistence wiring
- it gives a useful fallback even before real-delay is ready

### Step 4. Implement short-lived real-delay probing

Build the real-delay probe around a temporary Xray subprocess.

Recommended sequence:

- create a temp directory/file for the generated Xray config
- write a minimal runtime config with one outbound based on the chosen node
- bind a local inbound on a free port
- spawn `xray run -c <temp-config>`
- wait for readiness with a bounded timeout
- issue one proxied request through the local port
- measure request duration
- stop the child process and clean up temp artifacts

The readiness wait must be explicit. Do not assume the process is ready immediately after spawn.

This step is the point where Phase 3 deliberately consumes the minimal runtime foundation that would otherwise have been introduced later.

### Step 5. Persist history and latest summary together

When a test completes:

- insert one row into `connection_tests`
- update cached latest-result columns if those are added in this phase
- preserve failure details even when only one half of the test succeeds

Recommended persistence behavior:

- if TCP fails, still record the attempt and skip real-delay
- if TCP succeeds but real-delay fails, record both pieces accurately
- do not erase older history

### Step 6. Add repository helpers for future UX

Round out the DB layer with helpers that future phases will need.

Useful additions:

- list latest test summary joined onto configs
- get test history for one config
- optionally list configs ordered by latest real delay

Even if the CLI only uses a subset now, these helpers will make the HTTP API and TUI phases easier later.

### Step 7. Add focused tests

Add unit and integration coverage for:

- TCP success and failure classification where practical
- conversion from tester result to DB insert model
- repository latest-result behavior
- CLI parsing for `test`

Where Xray-dependent end-to-end testing is hard locally, keep the subprocess layer small enough that most logic can be tested without launching the real binary.

## Suggested Milestones

### Milestone A: TCP-only usable path

Complete when:

- `xrat test <id> --tcp-only` works
- results are written to `connection_tests`
- failures are classified consistently

This gives immediate practical value and de-risks the rest.

### Milestone B: Real-delay probe works for one config

Complete when:

- XRAT can launch a short-lived Xray subprocess
- one proxied request is measured successfully
- the result is persisted alongside the TCP result

### Milestone C: Latest-result summary is easy to read

Complete when:

- latest test data can be fetched cheaply per config
- future list/status views do not need custom SQL every time

## Testing Strategy

Phase 3 needs a layered test strategy.

### Unit tests

Focus on:

- failure classification
- DB model mapping
- CLI parsing
- small orchestration decisions such as skipping real-delay after TCP failure

### Repository tests

Use a temporary SQLite database to verify:

- connection test insertions
- latest-result retrieval
- config summary cache updates if added

### Subprocess boundary tests

Abstract the Xray runner so most orchestration can be tested with a fake process adapter.

That keeps tests stable even when the real Xray binary is unavailable in CI or local runs.

### Manual verification

Before declaring the phase done, manually confirm:

- a good config records both TCP and real-delay success
- an invalid host records a DNS or timeout failure
- a refused port records `refused`
- a broken Xray invocation records `process`

## Completion Criteria

Phase 3 can be considered complete when:

1. XRAT exposes `xrat test <id>`
2. a stored config can be loaded and TCP-tested from the CLI
3. a real-delay probe runs through actual proxy traffic rather than socket-open timing alone
4. each run is persisted into `connection_tests`
5. the latest known result is easy to fetch for later UI/API work
6. failures are classified and surfaced clearly enough for users to understand what happened
7. the new code is covered by focused unit/repository/CLI tests

## Recommended Delivery Order

To keep risk low, build this phase in the following order:

1. add tester domain types and CLI scaffolding
2. ship TCP-only testing end to end
3. add `Node` to temporary Xray config generation plus short-lived process helpers
4. persist results and expose latest-result reads cleanly
5. add short-lived Xray real-delay probing
6. harden failure classification and cleanup behavior
7. add any batch/list helpers that naturally fall out of the implementation

## Open Questions

These should be answered while implementing, but they should not block starting the phase:

- what exact target URL should the real-delay probe use by default?
- should latest-result caching live on `configs` or remain query-derived for now?
- should `xrat test <id>` automatically skip deleted/disabled configs or allow explicit override?
- should the first version support only one config id, or also `selected` as a convenience target?
- what timeout defaults feel reasonable for TCP connect, Xray startup, and proxied request time?
