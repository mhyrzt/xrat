# Current Work: Phase 3 & 4 Implementation

## Context

We've completed Phase 1 (subscription parsing), Phase 2 (database persistence), and Phase 2.5 (CLI foundation). The codebase now has:

- Working subscription parser for vless, vmess, ss, trojan, http, socks5
- SQLite persistence with migrations for configs, subscriptions, connection_tests, runtime_sessions
- Command-first CLI structure with `import`, `add`, `list` commands
- Modular architecture with separated concerns

## Current Goal

Implement Phase 3 (Connection Testing) and Phase 4 (Managed Xray Runtime) to give XRAT the ability to:

1. Test stored configs for connectivity and real latency
2. Run Xray as a managed long-lived proxy session

## Phase 3: Connection Testing

### What We're Building

- `xrat test <id>` command to validate stored configs
- ICMP ping check (fastest, most basic reachability test)
- TCP reachability check (fast, cheap connectivity test)
- Real-delay check through actual proxy traffic (not just socket timing)
- Short-lived Xray subprocess for probing
- Persist test results in `connection_tests` table
- Failure classification and clear error reporting

### Implementation Plan

1. **Xray config generation** (`src/xray/config.rs`)
   - Convert stored `Node` to runnable Xray JSON
   - Support temporary probe configs with ephemeral ports
   - Handle inbound/outbound generation for all protocols

2. **ICMP ping tester** (`src/tester/icmp.rs`)
   - Send ICMP echo request to target host
   - Measure round-trip time
   - Classify failures (unreachable, timeout, permission denied)

3. **Process management** (`src/xray/process.rs`)
   - Spawn Xray as child process
   - Wait for readiness (port listening)
   - Capture stdout/stderr
   - Clean shutdown and cleanup

3. **TCP tester** (`src/tester/tcp.rs`)
   - Socket connect with timeout
   - Measure connect time
   - Classify failures (DNS, timeout, refused, etc.)

4. **Real-delay tester** (`src/tester/real_delay.rs`)
   - Generate temp Xray config for one node
   - Start short-lived Xray process
   - Send HTTP request through local proxy
   - Measure total latency
   - Teardown process and temp files

5. **Test orchestration** (`src/tester/mod.rs`)
   - Coordinate TCP + real-delay tests
   - Persist results to database
   - Return structured test outcome

6. **CLI command** (`src/cli/test.rs` + `src/app/commands/test.rs`)
   - Parse `xrat test <id>` command
   - Load config from database
   - Run tests and display results

### Key Decisions

- Default test target: Use a lightweight HTTPS endpoint for real-delay (e.g., `https://www.gstatic.com/generate_204`)
- Timeout defaults: 2s for ICMP, 5s for TCP, 10s for real-delay
- Test order: ICMP → TCP → Real-delay (fail fast, skip expensive tests if basic connectivity fails)
- Process cleanup: Always kill child process and remove temp files, even on error
- Failure classification: DNS, timeout, refused, process, proxy, unknown

## Phase 4: Managed Xray Runtime

### What We're Building

- `xrat connect <id>` - start Xray with a stored config
- `xrat disconnect` - stop the running Xray session
- `xrat status` - show current runtime state
- Long-lived Xray process management
- Runtime session persistence in `runtime_sessions` table
- Clean switching between configs

### Implementation Plan

1. **Runtime config generation** (extend `src/xray/config.rs`)
   - Generate full runtime configs (not just probe configs)
   - Support SOCKS + HTTP inbounds on fixed/configurable ports
   - Handle routing and DNS settings

2. **Runtime manager** (`src/xray/runtime.rs`)
   - Track active session state
   - Manage long-lived Xray process
   - Handle startup, shutdown, switching
   - Persist session state to database

3. **Session repository** (extend `src/db/repository/runtime_sessions.rs`)
   - Insert/update session records
   - Track status transitions (starting → running → stopped/failed)
   - Query active session

4. **CLI commands**
   - `src/cli/connect.rs` + `src/app/commands/connect.rs`
   - `src/cli/disconnect.rs` + `src/app/commands/disconnect.rs`
   - `src/cli/status.rs` + `src/app/commands/status.rs`

### Key Decisions

- Single active session model (one config running at a time)
- Default ports: SOCKS on 1080, HTTP on 8080 (configurable later)
- Session states: starting, running, stopping, stopped, failed
- Auto-replace: `connect` stops existing session before starting new one
- PID tracking: Store process ID for status verification

## Delivery Order

### Phase 3 Steps

1. Create `src/xray/` module structure
2. Implement Xray config generation for single nodes
3. Implement ICMP ping tester
4. Implement process spawning and management
5. Implement TCP connectivity tester
6. Implement real-delay tester with temp Xray runtime
7. Wire up `xrat test <id>` command
8. Add tests for tester and persistence

### Phase 4 Steps

1. Extend config generation for full runtime configs
2. Implement runtime session manager
3. Implement `xrat connect <id>` command
4. Implement `xrat disconnect` command
5. Implement `xrat status` command
6. Add session state persistence
7. Add tests for runtime lifecycle

## Success Criteria

### Phase 3 Complete When

- `xrat test <id>` runs ICMP, TCP, and real-delay checks
- Results are persisted in `connection_tests`
- Failures are classified and reported clearly
- Temp Xray processes are cleaned up properly
- Tests cover tester logic and persistence

### Phase 4 Complete When

- `xrat connect <id>` starts a managed Xray session
- `xrat disconnect` stops the session cleanly
- `xrat status` shows current runtime state
- Session state is persisted in `runtime_sessions`
- No orphaned Xray processes after shutdown
- Tests cover runtime lifecycle

## Next Actions

Starting with Phase 3:

1. Create `src/xray/` module with config generation
2. Implement ICMP ping tester
3. Implement basic Xray JSON structure for outbounds
4. Add process spawning helpers
5. Build TCP tester
6. Build real-delay tester
7. Wire up CLI command

Let's begin!
