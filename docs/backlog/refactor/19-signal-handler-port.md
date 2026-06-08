# Add Signal Handler Port

## Finding

### [Priority: Low] Add a signal handler port for graceful shutdown

**Files involved:**

- `src/server/mod.rs:39`
- `src/app/commands/logs.rs:144`
- `src/app/commands/test/handlers/ping.rs:35`
- `src/xray/process_mgmt/signals.rs`

**Problem:** `tokio::signal::ctrl_c()` is called in 3 places for graceful
shutdown (HTTP server, log follow, ping cancel). Process signal sending
(SIGTERM, SIGKILL via `kill` command) is in `signals.rs`. None of these have
test seams.

**Why this change is needed:** Ctrl-C handlers cannot be tested. Signal-based
process termination cannot be tested without real system processes. Extracting a
port would enable shutdown and termination tests, but the value is low since
these paths are stable and rarely change.

**How to implement it:** Introduce a `SignalHandler` trait with methods for
shutdown notification and process signal sending. Provide a production
`OsSignalHandler` and a test no-op implementation. Inject into server and
process lifecycle code.

**Positive effect on the codebase:** Server shutdown and process termination
become testable. Signal handling is centralized rather than duplicated.

**Suggested target architecture:** `SignalHandler` port in `src/support/` or
`src/app/ports/`. Used by server, log follow, ping cancel, and process
termination.

**Risk / migration notes:** Very low risk. Consider deferring until
`ProcessSpawner` and runtime lifecycle ports are in place, since signal handling
is tightly coupled to process management.
