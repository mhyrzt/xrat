# Current Work: Phase 4 Implementation

## Context

We've completed Phase 1 (subscription parsing), Phase 2 (database persistence),
Phase 2.5 (CLI foundation), and Phase 3 (connection testing). The codebase now
has:

- Working subscription parser for vless, vmess, ss, trojan, http, socks5
- SQLite persistence with migrations for configs, subscriptions,
  connection_tests, runtime_sessions
- Command-first CLI structure with `import`, `add`, `list`, and `test` commands
- Connection testing with ICMP, TCP, and real-delay probing
- Short-lived Xray probe config generation and process management
- Modular architecture with separated concerns

## Phase 3 Status: Complete

Phase 3 is done.

Implemented:

- `xrat test <id>` command
- ICMP reachability check
- TCP connectivity check with explicit DNS classification
- Real-delay check through a temporary local Xray proxy
- Probe Xray config generation for stored nodes
- Short-lived Xray subprocess startup, readiness, and cleanup
- Persistence of test history in `connection_tests`
- Focused tests covering CLI parsing, node reconstruction, tester behavior, and
  process handling

Key outcomes:

- Stored configs can now be validated from the CLI
- Test runs persist history in SQLite
- Failures are classified into DB-compatible buckets
- Sandbox-sensitive probe tests now behave deterministically in restricted
  environments

## Current Goal

Implement Phase 3.5 (local app configuration) before Phase 4 so XRAT has a
clear runtime configuration surface for file-based settings such as
`~/.config/xrat/config.toml` and managed geo assets.

## Phase 3.5: Local App Configuration

### Why This Phase Exists

Phase 3 proved that XRAT can test stored configs, but Phase 4 needs stable
runtime inputs that should not live in SQLite rows or ad hoc CLI flags.

Before building managed long-lived sessions, we should define:

- what lives in `config.toml`
- how local routing and geo policy is stored and consumed
- which values are user-managed files versus database state
- how runtime defaults are resolved from disk

This keeps Phase 4 from hardcoding behavior that we will immediately need to
move into local configuration.

### What We're Going To Discuss

- `~/.config/xrat/config.toml` as the app-level settings file
- routing direct/block lists in `config.toml`
- `[geo]` profiles for local or remotely fetched `geosite.dat` and `geoip.dat`
- naming and path conventions under the XRAT config directory
- file creation behavior and default contents
- how these files feed future Xray runtime generation

### Proposed Scope

1. **Config file shape**
   - Define the TOML schema for user-managed runtime settings
   - Decide which fields belong here versus in SQLite
   - Add loader and validation behavior later once the schema is agreed

2. **Routing and geo policy**
   - Define direct/block route list shape for domains, IPs, geosite, and geoip
   - Define managed geo profiles with explicit `name`, `geosite`, and `geoip`
     sources
   - Allow geo sources to be local files or URLs

3. **Runtime path conventions**
   - Standardize file names under `~/.config/xrat/`
   - Use `config.toml` as the canonical app config filename
   - Decide whether `XRAT_PATH` keeps overriding the whole runtime directory

4. **Integration boundary for Phase 4**
   - Map config file values into generated Xray runtime config
   - Map direct/block entries into routing rules
   - Map DNS settings, hosts, and query strategy into generated Xray config
   - Keep DB-backed config records separate from local machine policy files

### Candidate `config.toml` Responsibilities

- local inbound settings such as SOCKS, HTTP, and local Shadowsocks
- runtime log settings
- runtime sniffing settings
- default connect behavior and runtime toggles
- test target URL overrides if we later want them configurable
- managed geo asset profiles and source URLs/files
- optional DNS and routing defaults for generated runtime configs

### Current `config.example.toml` Shape

- `[paths]` for `db.sqlite` and optional Xray/V2Ray binary paths
- `[runtime]` for engine choice and session replacement behavior
- `[runtime.log]`, `[runtime.socks]`, `[runtime.http]`,
  `[runtime.shadowsocks]`, and `[runtime.sniffing]`
- `[routing]`, `[routing.direct]`, and `[routing.block]`
- `[geo]` plus `[[geo.profiles]]` for managed `geosite.dat` and `geoip.dat`
  sources
- `[dns]` and `[dns.hosts]`
- `[testing.real_delay]`, `[testing.icmp]`, `[testing.download]`, and planned
  `[testing.tcp]`

### Open Questions

- should missing files be auto-created with defaults?
- should invalid route or geo entries fail startup, be skipped with warnings, or
  be reported only on demand?
- should geo profile files be fetched only on demand or proactively when
  `auto_update` is enabled?
- should planned test config fields stay in the schema before `xrat test`
  consumes `config.toml`?

### Success Criteria

- XRAT has a documented local config file story before Phase 4 runtime work
- the ownership boundary between SQLite state and local disk config is clear
- `config.toml` has an agreed first-pass format
- local geo asset behavior has an agreed first-pass format
- Phase 4 can consume these files without rethinking the runtime surface

## Phase 4: Managed Xray Runtime

### What We're Building

- `xrat connect <id>` - start Xray with a stored config
- `xrat disconnect` - stop the running Xray session
- `xrat status` - show current runtime state
- Long-lived Xray process management
- Runtime session persistence in `runtime_sessions` table
- Clean switching between configs

### Implementation Plan

1. **Runtime config generation** (`src/xray/config.rs`)
   - Extend config generation beyond probe mode
   - Support SOCKS and optional HTTP local inbounds
   - Reuse stored node fields for runnable Xray sessions

2. **Runtime manager** (`src/xray/runtime.rs`)
   - Start and stop long-lived Xray processes
   - Track process id and listening ports
   - Handle startup failure and cleanup

3. **Session repository flow** (`src/db/repository/runtime_sessions.rs`)
   - Insert runtime session rows
   - Track status transitions
   - Load latest and currently running sessions

4. **CLI commands**
   - `src/cli/connect.rs` + `src/app/commands/connect.rs`
   - `src/cli/disconnect.rs` + `src/app/commands/disconnect.rs`
   - `src/cli/status.rs` + `src/app/commands/status.rs`

### Key Decisions

- Single active session model
- Default ports: SOCKS on 1080, HTTP on 8080
- Session states: starting, running, stopping, stopped, failed
- `connect` replaces an existing active session cleanly
- Runtime state is persisted and also verified against process reality where
  useful

## Delivery Order

### Phase 3.5 Steps

1. Use `config.toml` as the canonical app config filename
2. Define the first-pass TOML schema
3. Define routing direct/block and geo profile formats
4. Decide file creation and validation behavior
5. Feed those decisions into Phase 4 runtime work

### Phase 4 Steps

1. Extend Xray config generation for runtime sessions
2. Add runtime process manager
3. Implement `xrat connect <id>`
4. Implement `xrat disconnect`
5. Implement `xrat status`
6. Persist runtime lifecycle transitions
7. Add focused tests for runtime lifecycle

## Success Criteria

### Phase 4 Complete When

- `xrat connect <id>` starts a managed Xray session
- `xrat disconnect` stops the session cleanly
- `xrat status` shows current runtime state
- Session state is persisted in `runtime_sessions`
- No orphaned Xray processes remain after shutdown
- Tests cover runtime lifecycle behavior

## Next Actions

Starting with Phase 3.5:

1. Use `config.toml` as the canonical app config filename
2. Implement an app config loader for the first-pass `config.toml` schema
3. Migrate `Config.toml` to canonical lowercase `config.toml`
4. Decide how strict XRAT should be when parsing route and geo entries
5. Use the loaded config to drive Phase 4 runtime implementation

Then move into Phase 4:

1. Extend runtime config generation in `src/xray/config.rs`
2. Add long-lived process management in `src/xray/runtime.rs`
3. Wire `connect`, `disconnect`, and `status` through CLI and app layers
4. Persist runtime state transitions in SQLite
5. Add focused runtime lifecycle tests
