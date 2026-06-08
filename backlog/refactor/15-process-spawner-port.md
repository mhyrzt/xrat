# Add Process Spawner Port

## Finding

### [Priority: High] Add a process spawner port for external subprocess calls

**Files involved:**

- `src/app/commands/daemon.rs`
- `src/app/commands/daemon_install.rs`
- `src/app/commands/proxy/desktop.rs`
- `src/app/commands/upgrade/source.rs`
- `src/app/commands/upgrade/mod.rs`
- `src/xray/process_mgmt/process.rs`
- `src/xray/process_mgmt/signals.rs`
- `src/xray/process/spawn.rs`
- `src/singbox/process_mgmt.rs`
- `src/prober/icmp/mod.rs`

**Problem:** `std::process::Command` is used directly in 10 production files for
spawning subprocesses: xray and sing-box runtime engines, ping for ICMP probing,
kill for process termination, systemctl for daemon install, gsettings for
desktop proxy, cargo for source upgrades, and the daemon binary itself. Each
call site builds its own command, configures its own stdio, and handles errors
independently.

**Why this change is needed:** Every module that spawns an external binary
requires it to be installed and on `$PATH` in tests. Failure paths (binary not
found, crash, hang, non-zero exit) are untested. The same process-lifecycle
patterns (spawn, check readiness, terminate, wait) are duplicated across
xray/process, xray/spawn, and singbox/process.

**How to implement it:** Introduce a `ProcessSpawner` trait with methods for
spawning, detecting, and terminating child processes. Include structured results
that capture exit status, stdout, and stderr. Provide a `SystemProcessSpawner`
production adapter and a `MockProcessSpawner` test adapter. Consolidate the
three engine startup implementations (xray managed, xray ad-hoc, sing-box)
behind a shared runtime-process abstraction that uses this port.

**Positive effect on the codebase:** Engine startup and teardown become testable
without real binaries. Failure scenarios (binary missing, crash on start, slow
startup, hang on stop) become reproducible. The three engine startup
implementations can share one polling loop (see `16-port-waiter`).

**Suggested target architecture:** `ProcessSpawner` port in application or
xray/singbox service layer; `SystemProcessSpawner` adapter; engine startup
orchestration uses `ProcessSpawner` + `PortWaiter`.

**Risk / migration notes:** Medium risk because process lifecycle is
user-visible. Start by extracting the spawning port alone without changing
engine startup behavior. Add a `MockProcessSpawner` for the existing
`ProcessInspector` tests first, then migrate CLI and TUI callers.

**Note:** This overlaps with `ProcessInspector` (which only does `/proc/` reads)
and `PortWaiter` (which handles TCP readiness polling). Consider unifying all
three into a single `RuntimeProcessManager` port.
