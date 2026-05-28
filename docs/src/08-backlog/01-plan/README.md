# xrat Plan

## Goal

Build a Rust-based TUI Xray client that can ingest subscriptions, validate and
store configs, run Xray with a working config, and expose config data through
simple HTTP endpoints.

## Assumptions

- Xray is already installed on the system.
- The app is primarily a local desktop/terminal tool.
- SQLite is the initial database backend.
- The HTTP server is intended for local or trusted-network use unless auth is
  added later.

## Current Status

- Subscription parsing is implemented in Rust.
- The parser currently supports `vless`, `vmess`, `ss`, `trojan`, `http`, and
  `socks5` links.
- The codebase is now split into multiple Rust modules.
- The CLI is now command-first and split into dedicated `src/cli/` modules.
- Command execution is now split into `src/app/commands/` with a thin
  `src/main.rs`.
- Core dependencies for TUI, async runtime, database, and HTTP server are
  present in `Cargo.toml`.

## Roadmap

### Phase 1: Subscription Ingestion

- Finalize parsing behavior for the supported link formats.
- Normalize parsed nodes consistently before persistence.
- Support importing from subscription URLs and raw subscription text.
- Deduplicate configs before saving them.

### Phase 2: Database Persistence

- Add a SQLite schema for configs and subscription sources.
- Save parsed configs into the database instead of only writing JSON files.
- Track metadata such as protocol, address, port, remark/name, source URL, and
  timestamps.
- Mark configs as active, disabled, or deleted without physically removing rows.
- Expose persistence-backed CLI commands such as:
  - `xrat import <input>`
  - `xrat add <config-uri>`
  - `xrat list configs`
  - `xrat list subscriptions`

Suggested tables:

- `subscriptions`
- `configs`
- `connection_tests`
- `runtime_sessions`

### Phase 2.5: CLI Foundation

- Move the CLI to a command-first shape: `xrat COMMAND [ARGS] [FLAGS]`.
- Keep shared/global flags such as `--database` and `--config`.
- Limit this phase to CLI structure plus commands already supported by existing
  persistence work.
- Avoid pulling later-phase commands forward just because the CLI can name them
  now.

This phase should mainly cover:

- `import`
- `add`
- `list`

Recommended structural outcome:

- `src/cli/` for Clap definition/parsing
- `src/app/commands/` for command behavior
- `src/app/runtime.rs` for shared runtime/bootstrap context
- `src/main.rs` as a minimal entrypoint

### Phase 3: Connection Testing

- Implement connectivity checks similar to other clients.
- Support lightweight TCP reachability checks.
- Add the minimal Xray runtime foundation needed for testing:
  - generate a temporary runnable Xray config from one stored node
  - launch a short-lived Xray child process for probing
  - wait for local proxy readiness and clean up processes/files reliably
- Measure real delay/latency using actual proxy traffic, not just socket open
  time.
- Save test results in the database with timestamps and per-config history.
- Keep the latest result cached for fast UI display.
- Add CLI commands related to testing once the behavior exists, such as:
  - `xrat test <id>`

Suggested metrics to store:

- TCP ping success/failure
- TCP connect time
- Real delay success/failure
- Real delay in milliseconds
- Last checked timestamp
- Failure reason

### Phase 4: Xray Core Execution

- Build on the runtime foundation from Phase 3 to manage a long-lived Xray
  session.
- Track running state, selected config, ports, and process metadata.
- Stop or restart Xray cleanly when switching configs.
- Save the last known working config and runtime state in the database.
- Add runtime-oriented CLI commands here, such as:
  - `xrat connect <id>`
  - `xrat disconnect`
  - `xrat status`

### Phase 5: HTTP API

- Run a lightweight Axum-based HTTP server alongside the app.
- Expose `/json` to return configs in JSON form.
- Expose `/b64` to return configs encoded as base64 subscription text.
- Expose `/health`, `/configs`, and `/configs/:id` for health checks and
  management-oriented metadata.
- Support a query parameter for returning the top `n` configs sorted by real
  delay.
- Support an optional `key` query parameter for simple request authentication.
- Start standalone through `xrat serve`; optionally start alongside the daemon
  when `server.enabled = true`.

### Phase 6: TUI Application

- Build the main interface with Ratatui.
- Show subscription sources, config list, test results, and runtime state.
- Allow importing, refreshing, testing, filtering, sorting, and selecting
  configs.
- Allow starting/stopping Xray and switching the active config.
- Surface logs and recent failures in a minimal diagnostics view.
- Bring richer config-management actions into the main UX here or in the phase
  that formalizes config management, such as:
  - `show`
  - `select`
  - `enable`
  - `disable`
  - `delete`
  - `restore`

## Proposed Architecture

- `src/parser/`: parse and normalize subscription lines.
- `src/model/`: shared domain types.
- `src/cli/`: Clap command/flag definitions and CLI parsing tests.
- `src/app/commands/`: command handlers for implemented CLI commands.
- `src/app/runtime.rs`: app bootstrap context and shared runtime paths.
- `src/app/input/`: input source reading and source classification.
- `src/support/`: shared helper code such as decoding.
- `src/db/`: database models, queries, and migrations.
- `src/tester/`: TCP ping and real-delay test logic.
- `src/xray/`: Xray config generation and process management.
- `src/server/`: Axum routes and API state.
- `src/tui/`: Ratatui app state, rendering, and input handling.
- `src/main.rs`: bootstrap CLI/app mode selection.

## Execution Order

1. Finish persistence layer and store parsed configs in SQLite.
2. Define schema and migrations for configs, tests, and runtime state.
3. Add TCP testing plus the minimal Xray runtime foundation needed for
   real-delay probes.
4. Add real-delay testing and persist test history/latest summaries in the
   database.
5. Add managed Xray runtime control using the Phase 3 runtime foundation.
6. Add the Axum server with `/json` and `/b64` endpoints.
7. Build the TUI around those core capabilities.

## Open Questions

- Should real-delay testing use a fixed target URL or a configurable endpoint?
- Should `/b64` output include all saved configs or only enabled/healthy ones?
- Should the `top n` filter use only healthy configs with valid real-delay
  results?
- Should the server expose only the latest test result or full test history?
- Is the optional `key` query parameter enough, or should auth move to headers
  later?
- Should Xray be run in global mode, local SOCKS/HTTP mode, or both?
- Should subscription refresh be manual first, or scheduled from the start?

## Near-Term Next Steps

- Continue improving the Phase 2.5 CLI foundation around `import`, `add`, and
  `list`.
- Add detailed read commands only when they clearly fit the active phase.
- Keep lifecycle, testing, and runtime commands inside their respective phases.
