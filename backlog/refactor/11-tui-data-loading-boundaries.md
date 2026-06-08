# Separate TUI Data Loading From Direct I/O And Process Probing

## Finding

### [Priority: Medium] Separate TUI data loading from direct I/O and process probing

**Files involved:**

- `src/tui/data/mod.rs`
- `src/tui/run/tasks/data.rs`
- `src/tui/run/tasks/version_check.rs`
- `src/tui/run/tasks/source.rs`
- `src/tui/run/tasks/runtime.rs`

**Problem:** `TuiData::load` performs repository queries, runtime status checks,
log loading, network address derivation, daemon IPC status, and view-model
construction in one function. `probe_engines` runs runtime binaries from the TUI
data module, and `version_check` performs direct HTTP requests with silent
`ok()?` failure paths.

**Why this change is needed:** TUI data loading should be a thin adapter over
application read models. Direct I/O inside TUI data modules makes update/render
tests harder and can freeze or silently degrade the UI if an external dependency
behaves unexpectedly.

**How to implement it:** Create `DashboardService` or `OverviewUseCase` in
application code to assemble configs, sources, runtime, tests, logs, daemon
info, and API URL facts. Move engine probing behind a `RuntimeEngineProbe` port.
Move latest-version checks behind an update service shared with the CLI
upgrade/update path. Keep TUI data types as view models converted from
application overview results.

**Positive effect on the codebase:** TUI update/render tests can use pure data
fixtures. Startup and refresh failures become easier to isolate, and the same
overview data can support HTTP or CLI status dashboards.

**Suggested target architecture:** TUI tasks call application overview/update
services; TUI app state stores view models; TUI views render only.

**Risk / migration notes:** Medium risk because TUI startup behavior is
user-visible. Extract read-only data assembly first, then move engine and
version probes behind ports.
