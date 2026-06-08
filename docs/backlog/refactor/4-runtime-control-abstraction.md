# Introduce Shared Runtime-Control Abstraction

## Finding

### [Priority: High] Introduce a runtime-control abstraction shared by CLI, TUI, and daemon

**Files involved:**

- `src/app/commands/connect.rs`
- `src/app/commands/disconnect.rs`
- `src/app/commands/status/mod.rs`
- `src/tui/run/tasks/runtime.rs`
- `src/app/runtime_service`
- `src/app/daemon/ipc`

**Problem:** Runtime operations are split across direct daemon IPC in CLI
commands and direct `RuntimeService` usage in TUI tasks and daemon supervisor
code. The CLI connect, disconnect, and status commands resolve socket paths,
call IPC functions, interpret daemon-unreachable errors, and format daemon
payloads themselves. The TUI starts and stops runtime sessions locally with
`RuntimeService`, which can diverge from daemon-managed behavior.

**Why this change is needed:** The project wants one runtime application core
with multiple thin adapters, but current adapters choose different control
paths. This creates inconsistent runtime semantics, duplicated
unreachable-daemon handling, and harder testing because IPC, process management,
and presentation are mixed together.

**How to implement it:** Create a `RuntimeControl` trait or enum-backed service
with methods `status`, `connect`, `disconnect`, and `replace`. Provide
implementations for daemon IPC control and local in-process control. Add a
factory that chooses the implementation from app settings and runtime mode.
Update CLI and TUI tasks to call `RuntimeControl` instead of raw IPC or raw
`RuntimeService`. Keep `RuntimeService` as the process/session core used by the
daemon/local implementation.

**Positive effect on the codebase:** Runtime behavior becomes consistent across
interfaces, and tests can inject a fake runtime controller without sockets or
subprocesses. Daemon unreachable messages and fallback policy become
centralized.

**Suggested target architecture:** `RuntimeService` owns local process/session
mechanics; `RuntimeControl` is the application-facing port; CLI, TUI, daemon
IPC, and future HTTP endpoints call the same control interface.

**Risk / migration notes:** Medium risk because runtime control is user-visible.
Start with status and disconnect, then migrate connect and replace. Preserve
existing CLI daemon behavior unless a setting explicitly selects local control.
