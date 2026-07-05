---
id: TASK-33
title: Trait Usage & Gap Analysis
status: Done
assignee: []
created_date: '2026-07-05 14:43'
updated_date: '2026-07-05 14:44'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-4
dependencies: []
priority: medium
ordinal: 13
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/3-ports/13-trait-usage-gap-analysis.md`

# Trait Usage & Gap Analysis

## Overview

Survey of existing trait usage across the codebase and identification of seams
where trait-based ports would improve testability. Based on codebase audit and
cross-referencing with backlog items #1-#12.

---

## Current Trait Usage

Only **4 custom traits** exist across `src/`:

| Trait              | Scope        | Purpose                                      | Implementations                                                         | Test doubles   |
| ------------------ | ------------ | -------------------------------------------- | ----------------------------------------------------------------------- | -------------- |
| `GeoIpLookup`      | `pub`        | Polymorphic GeoIP backend (country/city/ASN) | 6 (LocalMmdb, RemoteIpApi, RemoteIpWhois, RateLimited, Chained, Cached) | 3 hand-written |
| `ProcessInspector` | `pub(super)` | Runtime process reattachment verification    | 1 (SystemProcessInspector)                                              | 2 hand-written |
| `StartupErrorExt`  | private      | Attach stderr to AppError (Xray variant)     | 1 (AppError)                                                            | None           |
| `StartupErrorExt`  | private      | Same pattern for sing-box                    | 1 (AppError)                                                            | None           |

### `dyn Trait` usage

- **`dyn GeoIpLookup`**: 17 occurrences — the only trait used polymorphically
  across module boundaries via `Arc<dyn GeoIpLookup>`
- **`dyn ProcessInspector`**: 3 occurrences — scoped to
  `runtime_service/reattach`

### `impl Trait` in signatures

~27 occurrences, exclusively for standard library traits: `impl Into<String>`,
`impl AsRef<str>`, `impl FnOnce`, `impl Iterator`.

### Generic `<T>` / `where` clauses

- 2 generic structs: `DaemonResponse<T>`, `PaginatedResponse<T>`
- ~16 generic functions: 7 `map_*_row<R: Row>` for sqlx, 9 IPC/utility functions
- ~13 `where` clauses: mostly `R: Row` bounds for sqlx row mapping

### `#[async_trait]`

10 usages — all on `GeoIpLookup` and its implementations.

---

## Key Finding: No Port/Repository Traits

**Zero** repository, port, or use-case abstraction traits exist. Database
operations are concrete sqlx calls; filesystem, IPC, process, network, and clock
calls are direct. Adapter modules (CLI, HTTP, TUI) call concrete database and
I/O functions without any seam for fakes.

---

## Gap: Where Traits Are Missing

Each seam below corresponds to one or more backlog items where tests currently
require real databases, sockets, subprocesses, or filesystem access.

### 1. `ConfigRepository` (backlog #1, #2, #7)

**Problem:** CLI, HTTP, TUI all call `db::repository::configs::*` directly.
Config filtering, lifecycle mutations, and export behavior cannot be unit-tested
without a real database.

**Methods implied:**

```
list_configs(filter) -> Vec<Config>
get_config(id) -> Option<Config>
insert/subscription queries / lifecycle updates / soft-delete / hard-delete / restore
```

**Affected files:** `db/repository/configs/` (queries, import_ops, server_ops),
`app/commands/lifecycle.rs`, `app/commands/list.rs`, `app/commands/resolve.rs`,
`server/routes/configs.rs`, `server/routes/json.rs`, `server/routes/b64.rs`,
`tui/data/mod.rs`, `tui/data/configs.rs`

### 2. `RuntimeControl` (backlog #4, #5)

**Problem:** CLI commands use daemon IPC (`connect`/`disconnect`/`status` call
`ipc/` directly). TUI tasks call `RuntimeService` directly. No shared
abstraction means inconsistent runtime semantics and no fake for tests.

**Methods implied:**

```
status() -> RuntimeStatus
connect(config_id) -> Result
disconnect() -> Result
replace(config_id) -> Result
```

**Affected files:** `app/commands/connect.rs`, `app/commands/disconnect.rs`,
`app/commands/status/mod.rs`, `tui/run/tasks/runtime.rs`,
`app/runtime_service/`, `app/daemon/ipc/`,
`app/daemon/supervisor/handlers/runtime/`

### 3. `NetworkProbe` (backlog #3, #7)

**Problem:** Scanner and prober modules call TCP/ICMP/download/upload checks
directly. Tests cannot simulate latency, packet loss, or failures without real
network.

**Methods implied:**

```
tcp_check(addr, port) -> Result<Duration>
ping(addr) -> Result<Duration>
download_test(url) -> Result<Throughput>
upload_test(url) -> Result<Throughput>
real_delay(addr) -> Result<Duration>
```

**Affected files:** `prober/` (tcp, icmp, download, upload, real_delay),
`app/commands/test/`, `app/commands/scan/`

### 4. `DaemonClient` (backlog #4, #7)

**Problem:** IPC calls are concrete (`unix_impl.rs`). Daemon-unreachable
scenarios, timeout handling, and fallback behavior cannot be tested without a
real daemon socket.

**Methods implied:**

```
connect(config_id) -> DaemonResponse
disconnect() -> DaemonResponse
status() -> DaemonResponse<RuntimeStatus>
replace(config_id) -> DaemonResponse
```

**Affected files:** `app/daemon/ipc/client/unix_impl.rs`,
`app/daemon/ipc/client/unsupported_impl.rs`, `app/daemon/ipc/types.rs`,
`app/commands/connect.rs`, `app/commands/disconnect.rs`,
`app/commands/status/mod.rs`

### 5. `Filesystem` / `InputReader` (backlog #7)

**Problem:** `app_paths` reads env vars and writes config files; `source.rs`
writes JSON. Tests create temp dirs manually. No abstraction for file I/O.

**Methods implied:**

```
read_file(path) -> Result<String>
write_file(path, content) -> Result
path_exists(path) -> bool
config_dir() -> PathBuf
data_dir() -> PathBuf
```

**Affected files:** `app/context.rs`, `app/app_paths.rs`, `app/input/source.rs`,
`app/runtime_service/`

### 6. `Clock` (backlog #7)

**Problem:** Rotation cooldown, health timeouts, session timestamps use
`std::time` directly. Time-dependent behavior cannot be controlled in tests.

**Methods implied:**

```
now() -> DateTime<Utc>
elapsed(since) -> Duration
sleep(duration) -> Future
```

**Affected files:** `app/daemon/supervisor/handlers/runtime/`,
`app/runtime_service/replace_flow/`, `app/commands/test/settings/`

### 7. `EventRepository` (backlog #9)

**Problem:** Event recording uses concrete sqlx. Best-effort event persistence
cannot be asserted in tests without a real database.

**Methods implied:**

```
record_event(event) -> Result
list_events(filter) -> Vec<Event>
```

**Affected files:** `db/repository/events/`, `app/events.rs`,
`app/daemon/supervisor/`

### 8. `DashboardService` / `OverviewUseCase` (backlog #11)

**Problem:** `TuiData::load` performs repository queries + runtime probing + IPC
status + HTTP version check + address derivation in one function. No test seam
for any of those dependencies.

**Affected files:** `tui/data/mod.rs`, `tui/run/tasks/data.rs`,
`tui/run/tasks/version_check.rs`, `tui/run/tasks/source.rs`

---

## Additional Gaps (Not in Existing Backlog)

The following seams were discovered during a codebase-wide audit of direct I/O
calls. None are documented in existing backlog items.

### 9. `HttpClient` (cross-cutting)

**Problem:** `reqwest` is used in 8 production files outside of GeoIP lookups
(which are already abstracted behind `GeoIpLookup`). Upgrade downloads, TUI
version check, subscription imports, and all three probers (download, upload,
real-delay) construct their own `reqwest::Client` with no shared abstraction.

**Methods implied:**

```
get(url) -> Response
get_with_client(client, url) -> Response
head(url) -> Response
```

**Affected files:** `app/commands/upgrade/release.rs`,
`app/commands/geoip/download/executor.rs`, `tui/run/tasks/version_check.rs`,
`app/input/source.rs`, `config/import/subscription.rs`,
`prober/real_delay/check/request.rs`, `prober/download/check/proxied.rs`,
`prober/upload/request.rs`

**Testability impact:** Upgrade, version-check, subscription-import, and probe
tests all require real HTTP servers or network. Cannot simulate timeouts, bad
responses, or network failures.

---

### 10. `ProcessSpawner` (cross-cutting)

**Problem:** `std::process::Command` is used directly in 8 production files for
spawning subprocesses: xray, sing-box, ping, kill, systemctl, gsettings, cargo
build, and the daemon itself. `ProcessInspector` only covers zombie-read
(`/proc/` reads), not process creation or teardown.

**Methods implied:**

```
spawn(command, args) -> Child
spawn_with_output(command, args) -> Result<Output>
spawn_and_detach(command, args) -> Child
```

**Affected files:** `app/commands/daemon.rs`, `app/commands/daemon_install.rs`,
`app/commands/proxy/desktop.rs`, `app/commands/upgrade/source.rs`,
`app/commands/upgrade/mod.rs`, `xray/process_mgmt/process.rs`,
`xray/process_mgmt/signals.rs`, `xray/process/spawn.rs`,
`singbox/process_mgmt.rs`, `prober/icmp/mod.rs`

**Testability impact:** Every module that spawns an external binary requires it
to be installed and on `$PATH` in tests. Cannot test failure paths (binary not
found, crash, hang, non-zero exit).

---

### 11. `PortWaiter` (backlog #4, #7 overlap)

**Problem:** The same `TcpStream::connect` + `Instant::now` polling loop is
duplicated in three engine startup implementations. Identical logic for waiting
on a TCP port to become ready, with configurable timeout.

**Methods implied:**

```
wait_for_port(host, port, timeout) -> Result<Duration>
```

**Affected files:**

- `xray/process_mgmt/process.rs:74-99` (managed Xray)
- `xray/process/spawn.rs:63-83` (ad-hoc Xray)
- `singbox/process_mgmt.rs:89-113` (managed sing-box)

**Testability impact:** Startup timeout behavior cannot be tested without
binding actual TCP ports. A shared abstraction would also unify the duplicated
polling logic.

---

### 12. `DnsResolver` (backlog #3, #7 overlap)

**Problem:** `tokio::net::lookup_host` is called directly in TCP and ICMP
probers. DNS failures and resolution latency are untestable.

**Methods implied:**

```
lookup_host(hostname) -> Result<Vec<IpAddr>>
```

**Affected files:** `prober/tcp/check.rs:11`, `prober/icmp/mod.rs:43`

**Testability impact:** Cannot simulate DNS failures, slow resolution, or empty
results without real DNS queries.

---

### 13. `LocalIpResolver` (no backlog item)

**Problem:** The same `UdpSocket::bind("0.0.0.0:0")` + `connect("8.8.8.8:80")`

- `local_addr()` trick is used in two places to discover the primary LAN IP. One
  is a shared helper, the other duplicates the logic inline.

**Methods implied:**

```
primary_ip() -> Option<IpAddr>
```

**Affected files:** `support/net.rs:13-15`,
`app/commands/runtime_output.rs:22-25`

**Testability impact:** Low (logic is trivial), but the duplication itself is
unnecessary.

---

### 14. `SignalHandler` (no backlog item)

**Problem:** `tokio::signal::ctrl_c()` is used in 3 places for graceful shutdown
(server, logs follow, ping cancel). Process signal sending (`kill`, SIGTERM,
SIGKILL) is spread across `signals.rs` and `process_mgmt`.

**Methods implied:**

```
wait_for_shutdown() -> Future<()>
send_signal(pid, signal) -> Result
```

**Affected files:** `server/mod.rs`, `app/commands/logs.rs`,
`app/commands/test/handlers/ping.rs`, `xray/process_mgmt/signals.rs`

**Testability impact:** Low to medium. Ctrl-C handlers are hard to test but
seldom changed.

---

### 15. `PlatformDetector` (no backlog item)

**Problem:** `cfg!(target_os)` and `std::env::consts::ARCH` are scattered across
upgrade binary detection, ICMP ping flags, and desktop proxy gsettings
invocation.

**Methods implied:**

```
os() -> Os
arch() -> Arch
```

**Affected files:** `app/commands/upgrade/release.rs`,
`app/commands/proxy/desktop.rs`, `prober/icmp/mod.rs`

**Testability impact:** Low. Platform detection is trivial and rarely tested.

---

### 16. `Clipboard` (no backlog item)

**Problem:** `arboard::Clipboard::new()` + `set_text()` is called directly in
the TUI share task. The clipboard is a system dependency that cannot be mocked.

**Methods implied:**

```
copy_text(text) -> Result
```

**Affected files:** `tui/run/tasks/share.rs:6,83-103`

**Testability impact:** Low. TUI share is a minor feature. Inclusion here is for
completeness.

---

### 17. `EnvVars` (backlog #7 partial)

**Problem:** Beyond `app_paths` and `secret.rs`, 5 production locations read
environment variables directly: `SHELL`, `HTTP_PROXY`, `XDG_CURRENT_DESKTOP`,
`DESKTOP_SESSION`, `XDG_CONFIG_HOME`, `HOME`, `NO_COLOR`.

**Methods implied:**

```
get(key) -> Option<String>
```

**Affected files:** `app/commands/proxy/shell.rs`,
`app/commands/proxy/desktop.rs`, `app/commands/daemon_install.rs`,
`app/commands/output.rs`, `app/config/secret.rs` (already abstracted)

**Testability impact:** Low. `secret.rs` already shows the pattern (injectable
closure). Others are minor but inconsistently handled.

---

## Summary

| Seam               | Priority | Backlog ref          | Test doubles needed       | Affected files |
| ------------------ | -------- | -------------------- | ------------------------- | -------------- |
| `ConfigRepository` | High     | #1, #2, #7           | Fake DB / in-memory store | ~25            |
| `RuntimeControl`   | High     | #4, #5               | Fake runtime controller   | ~15            |
| `NetworkProbe`     | High     | #3, #7               | Stubbed network responses | ~8             |
| `ProcessSpawner`   | High     | new                  | Fake process factory      | ~10            |
| `HttpClient`       | High     | new                  | Fake HTTP server / client | ~8             |
| `PortWaiter`       | Medium   | new (overlap #4, #7) | Fake TCP listener         | ~3             |
| `DaemonClient`     | Medium   | #4, #7               | Fake daemon client        | ~8             |
| `Filesystem`       | Medium   | #7                   | In-memory FS              | ~5             |
| `Clock`            | Medium   | #7                   | Controllable clock        | ~5             |
| `EventRepository`  | Medium   | #9                   | In-memory event store     | ~5             |
| `DashboardService` | Medium   | #11                  | Fake all downstream ports | ~5             |
| `DnsResolver`      | Low      | new (overlap #3, #7) | Stub DNS resolver         | ~2             |
| `SignalHandler`    | Low      | new                  | No-op signal handler      | ~4             |
| `LocalIpResolver`  | Low      | new                  | Fake socket               | ~2             |
| `PlatformDetector` | Low      | new                  | Fixed os/arch             | ~3             |
| `EnvVars`          | Low      | new (overlap #7)     | Test env map              | ~4             |
| `Clipboard`        | Low      | new                  | No-op clipboard           | ~1             |

**Total affected files: ~100+** across `src/`, plus new port and service files.

The codebase has a mature trait pattern in `GeoIpLookup` (port, decorators,
factory, test doubles) but uses it nowhere else. Incremental extraction starting
with `ConfigRepository` and `RuntimeControl` would unblock the highest-priority
use-cases and cover the widest set of adapters (CLI + TUI + HTTP + daemon).
<!-- SECTION:DESCRIPTION:END -->
