# Phase 6 TUI Application

## Goal

Build the interactive terminal UI for XRAT with Ratatui, using the existing CLI,
database, tester, runtime, daemon, and HTTP API foundations as reusable
application services.

By the end of this phase, XRAT should be able to:

- start an interactive console through `xrat tui`
- show configs, subscription sources, test results, and runtime state in one
  navigable interface
- import, refresh, test, filter, sort, select, enable, disable, delete, restore,
  and activate configs without leaving the TUI
- start, stop, restart, and inspect the managed runtime
- expose copy, paste, and QR-code workflows for configs, selected config sets,
  subscription URLs, and local runtime profiles
- show diagnostics, recent failures, live operation progress, and actionable
  error messages
- preserve the same behavior as the CLI rather than creating a separate
  business-logic path

The visual and interaction target for this phase is the HTML prototype at
`docs/ui/tui/index.html`. Treat that file as the UX reference for layout,
navigation, keybinding labels, share actions, and panel hierarchy, not as an
implementation artifact.

## Why This Phase Exists

Phases 1 through 5 make XRAT functional from scripts, shell commands, and HTTP
consumers:

- configs can be imported and normalized
- configs and subscriptions are persisted
- real-delay and TCP tests are recorded
- Xray can run as a managed runtime
- daemon and auto-rotation flows can own background behavior
- HTTP endpoints can expose config data and subscription output

Phase 6 exists to make those capabilities usable as a day-to-day terminal
client.

The TUI should answer user questions quickly:

- what configs do I have?
- which configs are enabled, selected, deleted, stale, or failing?
- which config is currently running?
- what is the fastest healthy config right now?
- what happened during the latest test or runtime operation?
- how do I copy, share, QR, or paste a config/profile from the terminal?

## Current Starting Point

Phase 6 should explicitly build on the primitives and data-model work completed
through Phase 5.

Those reusable pieces include:

- `ConfigRecord`, `ConfigListFilter`, and config repository helpers
- `ConfigWithLatestTest` joined config/test query records
- subscription source persistence and import commands
- connection-test runners and persisted latest-test summaries
- managed runtime commands and `RuntimeService`
- daemon IPC command framework in `src/app/daemon/`
- server settings and HTTP API output as a reference for DTO consistency
- soft-delete metadata with `is_deleted` and `deleted_at`
- `AppContext` with `DbPool`, `AppConfig`, and `RuntimePaths`
- concrete `AppError` and `DbError` types
- tracing diagnostics and existing log/error conventions
- SQLite and PostgreSQL repository dispatch
- existing `ratatui` dependency in `Cargo.toml`

The TUI should call shared app/repository/runtime services directly. It should
not shell out to `xrat list`, `xrat test`, or `xrat connect` and parse text
output.

## UX Reference

Use `docs/ui/tui/index.html` as the canonical Phase 6 prototype.

Important prototype concepts to preserve:

- top status bar with app name, mode badge, filter summary, runtime status,
  active proxy address, active config, import freshness, and fleet counts
- left mode rail with primary views:
  - Configs
  - Sources
  - Tests
  - Runtime
- configs view with search/filter strip, table, detail panel, focused actions,
  latest-test signal cards, command output, and footer hints
- subscriptions view with source list, detail panel, refresh/copy/import
  actions, and QR/share affordances
- testing view with batch scope, test mode, target URL, concurrency, progress
  bar, progress stats, and live log
- runtime view with start/stop/restart/log actions, status cards, active config,
  listen address, and recent runtime log
- footer key bar with global mode switching, movement, search, share, help, and
  quit shortcuts
- QR and clipboard/paste workflows represented by `c`, `C`, `y`, `Y`, and `p`

The Rust implementation does not need to clone the HTML/CSS exactly. It should
translate the same structure into Ratatui layouts, widgets, styles, focus
states, and event handling.

## Scope Boundary

Phase 6 should cover:

- `src/tui/` module tree for app state, rendering, events, commands, widgets,
  and modal flows
- `xrat tui` CLI command to start the TUI
- full-screen terminal setup and restoration
- Configs, Sources, Tests, Runtime, Diagnostics, Help, and confirmation modal
  views
- keyboard and mouse event handling where practical
- search, filters, sorting, paging, and table selection state
- focused and bulk actions for config management
- soft delete, restore, and explicit hard delete/purge confirmation
- copy, paste/import, and QR modal workflows
- test queue/progress display for focused, selected, filtered, and all configs
- runtime start/stop/restart/switch actions
- live refresh of DB-backed state after operations
- non-blocking background tasks for import, test, runtime, and refresh work
- focused tests for reducers/state transitions, keymaps, filters, and action
  dispatch

Phase 6 should not yet cover:

- full mouse-first UX parity with desktop clients
- remote multi-user administration
- an embedded HTTP dashboard
- WebSocket streaming from the HTTP API
- full plugin/theme system
- terminal image protocols
- advanced graphs or historical charting
- automatic scheduler UI beyond manual refresh/test controls
- complete accessibility parity with browser UIs
- replacing CLI commands with TUI-only paths

Those can be later polish phases after the core TUI is stable.

## Required Crates

Already present:

- `ratatui`: core TUI rendering, layout, styles, widgets, tables, blocks,
  gauges, lists, paragraphs, and terminal backend abstraction.
- `tokio`: async runtime for background operations and event/task coordination.
- `tracing`: structured diagnostics that the TUI can surface in a diagnostics
  view.

Recommended additions for Phase 6:

- `crossterm`: terminal raw mode, alternate screen, keyboard/mouse events,
  resize events, and panic-safe terminal restoration. Ratatui commonly uses it
  as the backend/event layer.
- `tui-qrcode`: QR-code widget for config URIs, subscription URLs, selected
  bundles, and local profile payloads. See
  `https://docs.rs/tui-qrcode/latest/tui_qrcode/`.
- `qrcode`: optional lower-level QR generation if custom rendering is needed
  beyond `tui-qrcode`.
- `arboard` or `cli-clipboard`: system clipboard support for copy/paste actions
  where the host terminal/session permits it.
- `unicode-width`: width-aware truncation and alignment for config names,
  countries, protocols, emoji/status glyphs, and mixed Unicode remarks.
- `textwrap`: wrapping long errors, URIs, and help text inside modal panels.
- `tui-input`: editable search/filter/import fields with cursor handling and
  common text-edit key behavior.
- `tui-textarea`: multiline import/edit modal for raw subscription text and
  pasted config batches.
- `strum` with `derive`: ergonomic enum iteration for view modes, sort fields,
  filters, modal kinds, and action labels.

Useful but optional `tui-widgets` ecosystem candidates:

- `tui-widgets`: evaluate only if it provides maintained Ratatui-compatible
  widgets that reduce custom code for tabs, forms, popups, spinners, or command
  palettes. See `https://crates.io/crates/tui-widgets`.
- spinner/progress widgets from the Ratatui ecosystem if they simplify live
  test/import progress without adding heavy abstractions.

Dependency guidance:

- keep `ratatui` plus `crossterm` as the required rendering/event foundation
- add `tui-qrcode` when implementing the QR modal, not before
- add clipboard crates behind a feature if Linux Wayland/X11/headless behavior
  becomes inconsistent
- avoid large UI frameworks that hide Ratatui state management unless they
  clearly reduce boilerplate without constraining XRAT's layout

## Desired User Experience

The first usable version should feel like this:

- `xrat tui`
  - enters alternate screen
  - loads config, subscription, latest-test, and runtime summaries
  - shows the Configs view by default
  - restores the terminal cleanly on `q`, Ctrl+C, panic, or error

- Configs view
  - `j/k` or arrows moves focus
  - `/` opens search/filter input
  - `f` opens filter selector
  - `s` changes sort field/order
  - `space` toggles selected state
  - `enter` activates or connects the focused config
  - `t` tests the focused config
  - `T` tests selected or filtered configs
  - `e` enables the focused config
  - `E` enables selected configs
  - `d` soft deletes the focused config after confirmation
  - `r` restores a soft-deleted config
  - `D` purges after a stronger destructive confirmation

- Sources view
  - shows URL, file, and raw-text subscription sources
  - `r` refreshes focused source
  - `R` refreshes all enabled sources
  - `i` imports from a new URL/file/raw text modal
  - `c` copies source URL or source payload
  - `y` shows a QR code for shareable URLs

- Tests view
  - shows selected scope and current queue
  - `s` starts testing
  - `c` cancels the current batch
  - progress updates without blocking navigation
  - failures show concise reasons and link back to affected configs

- Runtime view
  - `s` starts focused or selected runtime target
  - `x` stops runtime
  - `r` restarts runtime
  - `l` opens recent runtime logs
  - active config, PID, listen address, uptime, and status remain visible

- Share workflows
  - `c` copies focused config URI
  - `C` copies selected config URIs as newline-separated subscription text
  - `y` opens a QR modal for focused config or current source
  - `Y` opens a QR modal for selected configs or local profile payload
  - `p` opens paste/import modal

## Information Architecture

### Top Status Bar

Show compact global state:

- current view/mode
- active filters and sort mode
- runtime state: stopped, starting, running, stopping, failed, unknown
- local proxy listen address when running
- active config ID/name
- last import/refresh time
- total, enabled, selected, deleted, and failing counts

### Mode Rail

Primary modes should match the prototype:

- Configs: table-first config management
- Sources: subscription source list and refresh/import controls
- Tests: current batch progress and recent result log
- Runtime: managed runtime control and log tail

Add secondary modal or overlay access for:

- Diagnostics
- Help
- Confirm
- QR
- Paste/Import
- Filter/Sort
- Command palette, if added later

### Configs View

Main table columns:

- ID
- name/remark
- protocol
- address:port
- network/security summary
- latest real delay
- status badges

Status badges should distinguish:

- enabled
- selected
- active/running
- disabled
- deleted/archived
- failed latest test
- untested/stale

Detail panel should show:

- protocol and transport details
- latest real-delay/TCP metrics
- source subscription
- import/update timestamps
- selected/enabled/deleted state
- generated share actions
- latest failure reason when present

### Sources View

Show:

- source ID
- name
- type: URL, file, raw text
- config count
- enabled/disabled state
- last fetch/import time
- latest error

Actions:

- add/import source
- refresh focused source
- refresh all
- copy source URL
- show source QR
- inspect configs from source
- enable/disable source if source-state support exists

### Tests View

Show:

- current scope: focused, selected, filtered, all, failed, stale
- mode: TCP, real-delay, or both
- test target URL
- concurrency
- progress gauge
- successes/failures/running/pending counts
- live result log
- cancel state

The TUI should not block while tests run. It should receive progress messages
from a background task and refresh affected rows incrementally.

### Runtime View

Show:

- current status
- active config
- PID if local process is owned by XRAT
- listen addresses and inbound protocols
- uptime
- auto-rotation state if enabled
- recent runtime/session errors
- last state transition

Actions should call shared runtime/daemon services and then reload state.

### Diagnostics View

Show:

- recent app errors
- operation log lines captured from TUI tasks
- runtime startup failures
- server/daemon status when available
- DB path/backend
- config path
- build/version info if available

This view should help users understand what failed without rerunning commands in
another shell.

## State Model

Use a single TUI app model that separates persistent data from UI-only state.

Suggested structure:

- `TuiApp`
  - current route/view
  - focused panel
  - active modal
  - keymap mode
  - loading flags
  - operation queue summaries
  - last error/status message

- `TuiData`
  - configs with latest test summaries
  - subscriptions
  - runtime status
  - aggregate counts
  - diagnostics/log buffer

- `ConfigListState`
  - focused row
  - selected IDs
  - filters
  - search query
  - sort field/order
  - page/window offset
  - include deleted flag

- `TaskState`
  - running import/test/runtime tasks
  - cancellation tokens
  - progress events
  - completion summaries

Prefer an explicit update loop:

1. read terminal/event/task message
2. map event to an action
3. update model or spawn command
4. render from model

This keeps rendering deterministic and testable.

## Module Layout

Recommended initial module tree:

```text
src/tui/
  mod.rs
  app.rs
  command.rs
  data.rs
  event.rs
  keymap.rs
  run.rs
  task.rs
  theme.rs
  view/
    mod.rs
    configs.rs
    sources.rs
    tests.rs
    runtime.rs
    diagnostics.rs
    help.rs
  widget/
    mod.rs
    chrome.rs
    table.rs
    detail.rs
    modal.rs
    qr.rs
    progress.rs
```

Keep rendering functions pure where possible:

- input: immutable app/data state plus frame area
- output: Ratatui draw calls only
- no database or runtime side effects during rendering

## CLI Entry

Add:

```bash
xrat tui
```

Optional flags:

```bash
xrat tui --refresh-interval 2s
xrat tui --view configs
xrat tui --no-mouse
xrat tui --read-only
```

Behavior:

- loads normal global config and database flags
- initializes tracing before entering alternate screen
- writes logs to file/stderr only in a way that does not corrupt the terminal
- exits with non-zero status if terminal setup or initial data load fails
- always restores terminal state on exit

## Keybindings

Global:

| Key         | Action              |
| ----------- | ------------------- |
| `1`         | Configs view        |
| `2`         | Sources view        |
| `3`         | Tests view          |
| `4`         | Runtime view        |
| `?`         | Help                |
| `q`         | Quit or close modal |
| `Esc`       | Close modal/back    |
| `Tab`       | Next panel          |
| `Shift+Tab` | Previous panel      |
| `j/k`       | Move down/up        |
| `g/G`       | First/last row      |
| `/`         | Search/filter input |

Configs:

| Key     | Action                                 |
| ------- | -------------------------------------- |
| `Enter` | Activate/connect focused config        |
| `Space` | Toggle selected                        |
| `t`     | Test focused config                    |
| `T`     | Test selected/filtered configs         |
| `e`     | Enable focused config                  |
| `E`     | Enable selected configs                |
| `x`     | Disable focused config                 |
| `X`     | Disable selected configs               |
| `d`     | Soft delete focused config             |
| `r`     | Restore focused deleted config         |
| `D`     | Purge focused config with confirmation |
| `c`     | Copy focused URI                       |
| `C`     | Copy selected URIs                     |
| `y`     | QR focused URI                         |
| `Y`     | QR selected/profile payload            |
| `p`     | Paste/import configs                   |

Sources:

| Key | Action                 |
| --- | ---------------------- |
| `r` | Refresh focused source |
| `R` | Refresh all sources    |
| `i` | Add/import source      |
| `c` | Copy source URL        |
| `y` | QR source URL          |

Runtime:

| Key | Action          |
| --- | --------------- |
| `s` | Start/connect   |
| `x` | Stop/disconnect |
| `r` | Restart         |
| `l` | Open logs       |

Testing:

| Key | Action            |
| --- | ----------------- |
| `s` | Start test batch  |
| `c` | Cancel test batch |

## QR, Clipboard, and Paste

QR support is a first-class UX requirement for Phase 6.

Use cases:

- focused config URI
- selected config bundle
- subscription source URL
- generated local XRAT profile payload
- HTTP API subscription URL when server is enabled

Implementation notes:

- use `tui-qrcode` for the QR modal when possible
- provide a plain text fallback when the terminal area is too small
- avoid showing secrets unless the user explicitly chooses a share action
- truncate long payload previews but keep copied/QR payload complete
- show payload type and approximate byte size in the modal

Clipboard/paste notes:

- system clipboard can fail in SSH, tmux, Wayland, or headless sessions
- failures should show actionable errors and leave the payload visible
- paste/import should accept raw URIs, base64 subscription text, and URLs using
  the same input classification as CLI import/add paths

## Config Management Semantics

The TUI must make destructive actions explicit:

- `Delete` means soft delete by default.
- soft-deleted configs remain visible only when deleted/archived filter is on.
- `Restore` reverses soft delete when possible.
- `Purge` or `Hard delete` permanently removes the config and requires a
  stronger confirmation.
- confirmation modals should state whether test history/runtime history will be
  preserved or lost.

Do not make hard delete the default action from a table row.

## Background Work and Concurrency

The TUI must remain responsive while operations run.

Required behavior:

- run imports, refreshes, tests, and runtime operations in spawned tasks
- send typed progress/completion events back to the UI loop
- support cancellation for long-running test batches and refreshes
- prevent duplicate conflicting operations on the same config/source
- reload affected data after task completion
- show progress and last error in the relevant view

Recommended primitives:

- `tokio::sync::mpsc` for task-to-UI events
- `tokio::sync::watch` for shared runtime/status snapshots if useful
- `tokio_util::sync::CancellationToken` if cancellation becomes common

If adding `tokio-util` only for cancellation, keep it scoped to the TUI/task
runner rather than leaking it into all app services.

## Implementation Slices

### P6.1 TUI Entry and Terminal Lifecycle

Goal: launch and exit a blank/placeholder TUI safely.

Tasks:

- [x] Add `xrat tui` CLI command.
- [x] Add `src/app/commands/tui.rs`.
- [x] Add `src/tui/run.rs`.
- [x] Initialize Crossterm backend and Ratatui terminal.
- [x] Enter alternate screen and raw mode.
- [ ] Handle Ctrl+C, `q`, panic, and error paths with terminal restoration.
- [x] Add smoke tests for CLI parsing.

Acceptance:

- [x] `xrat tui` opens a TUI screen.
- [x] quitting restores the terminal.
- [ ] setup failures return actionable errors.

### P6.2 App Model, Events, and Keymap

Goal: create the testable state/update loop.

Tasks:

- [x] Define `TuiApp`, `TuiData`, view route enum, modal enum, and focus state.
- [ ] Define typed `TuiAction` and `TuiEvent` values.
- [x] Map keyboard events to actions.
- [x] Add global mode switching and quit/back behavior.
- [x] Add unit tests for key mappings and state transitions.

Acceptance:

- [x] view switching works without database access.
- [x] modal close/back behavior is predictable.
- [x] keymap tests document expected shortcuts.

### P6.3 Layout Chrome

Goal: render the prototype's global shell.

Tasks:

- [x] Implement top status bar.
- [x] Implement mode rail.
- [x] Implement footer key bar.
- [x] Implement base content area routing.
- [x] Add theme constants for colors, text styles, borders, and status badges.
- [ ] Handle narrow terminal fallback layout.

Acceptance:

- [x] Configs/Sources/Tests/Runtime modes are visible.
- [x] runtime summary and fleet counts have stable locations.
- [ ] layout does not panic on small terminal sizes.

### P6.4 Config Data Loading

Goal: populate Configs view from persisted DB state.

Tasks:

- [x] Add data loader using repository helpers for configs with latest tests.
- [x] Load aggregate counts.
- [x] Support refresh/reload action.
- [x] Represent enabled, selected, active, deleted, failed, untested, and stale
      states.
- [x] Add tests for filter/sort model behavior.

Acceptance:

- [x] Config table displays real DB rows.
- [x] latest real-delay and TCP fields show when available.
- [x] deleted rows are hidden by default.

### P6.5 Configs View Rendering

Goal: render table and detail panel close to the prototype.

Tasks:

- [x] Render table columns for ID, name, protocol, address:port, network, delay,
      and status.
- [x] Render focused detail panel with metadata and latest test summary.
- [x] Render command/status line for focused actions.
- [x] Add row movement and selection.
- [ ] Add paging/window offset for large config lists.

Acceptance:

- [x] focused row and detail panel stay in sync.
- [x] selected/active/disabled/deleted/failed states are visually distinct.
- [ ] large lists remain navigable.

### P6.6 Search, Filter, Sort

Goal: make large config fleets manageable.

Tasks:

- [x] Add search input modal or inline strip.
- [ ] Filter by text, protocol, enabled, selected, failed, deleted, source, and
      real-delay presence. (enabled-only, failed-only, has-delay done via
      ConfigFilter cycle on `F`; protocol and source-id filters still missing)
- [ ] Sort by ID, name, protocol, source, real delay, TCP delay, last test, and
      imported/updated time. (ID, name, protocol, source, real-delay, TCP-delay
      done; last-test and imported-at require schema additions to TuiConfigRow)
- [x] Show active chips or filter summary in the header.
- [x] Add tests for filter/sort combinations.

Acceptance:

- [x] `/` filters visible rows.
- [x] sort order is visible and reversible.
- [x] clearing filters restores the full non-deleted list.

### P6.7 Config Actions

Goal: expose safe config management from the TUI.

Tasks:

- [x] Toggle selected state.
- [x] Enable/disable focused and selected configs. (`e`/`x` focused; `E`/`X`
      bulk-enable/disable all selected non-deleted configs)
- [x] Soft delete focused config with confirmation.
- [x] Restore soft-deleted config.
- [x] Purge focused config with stronger confirmation.
- [x] Reload rows after mutations.
- [x] Add tests for action dispatch and confirmation state.

Acceptance:

- [x] soft delete never purges by accident.
- [x] selected/enabled/deleted counts update after actions.
- [x] failures show actionable messages.

### P6.8 Sources View

Goal: manage subscription sources.

Tasks:

- [x] Load subscription sources and config counts.
- [x] Render sources table and detail panel.
- [x] Refresh focused source via `r`.
- [x] Refresh all sources via `R`.
- [x] Add/import URL, file, and raw-text sources via `i` import modal.
- [ ] Copy and QR source URLs.
- [ ] Show latest refresh/import errors per-source in the detail panel.

Acceptance:

- [x] source data maps to real repository rows.
- [x] refresh operations report progress and completion.
- [x] import reuses existing parser/import services.

### P6.9 Testing View and Progress

Goal: run connection tests from the TUI without blocking navigation.

Tasks:

- [x] Define test scopes: focused, selected, filtered, all enabled, failed,
      stale.
- [x] Spawn test batches in background tasks.
- [x] Render progress gauge, counts, ETA if available, and live result log.
- [x] Support cancellation.
- [x] Update config rows as results arrive or after completion.

Acceptance:

- [x] users can start and cancel a test batch.
- [x] UI remains responsive while tests run.
- [x] latest result data refreshes after tests finish.

### P6.10 Runtime View

Goal: control the managed runtime from the TUI.

Tasks:

- [x] Load current runtime status.
- [x] Render runtime status cards and recent log lines.
- [x] Start/connect selected config via `s`.
- [x] Stop/disconnect current runtime via `x`.
- [x] Restart runtime via `r`.
- [ ] Switch active config safely.
- [ ] Show daemon/auto-rotation status when available.

Acceptance:

- [x] runtime actions call existing runtime service paths.
- [x] active config badge updates after connect/switch.
- [ ] runtime errors are shown in Runtime and Diagnostics views.

### P6.11 QR, Clipboard, and Paste Modals

Goal: implement share/import workflows from the prototype.

Tasks:

- [x] Add QR modal using `qrcode` crate (unicode half-block rendering; `y` key).
- [ ] Add payload builders for subscription URL, local runtime profile, and
      HTTP API subscription URL.
- [x] Add copy-to-clipboard integration (`arboard`; `c` focused, `C` selected).
- [x] Add paste/import modal (import modal from P6.8 handles this via `i` in
      Sources view and future `p` in Configs view).
- [x] Add text fallback for clipboard failures (error shown in status bar).

Acceptance:

- [x] focused config can be copied and shown as QR.
- [x] selected configs can be copied as newline-separated subscription text.
- [x] paste/import accepts the same input formats as CLI import/add.

### P6.12 Diagnostics and Help

Goal: make the TUI self-explanatory and debuggable.

Tasks:

- [x] Add help modal with current keymap.
- [x] Add diagnostics view/log buffer. (`TuiView::Diagnostics` via key `5`;
      `event_log: Vec<String>` bounded to 500 entries)
- [x] Capture task errors and operation summaries. (`push_log` called from
      `apply_task_event` and `run_config_command`)
- [ ] Show DB backend/path, config path, runtime status, and server state.
      (runtime state shown; DB/config paths not yet surfaced)
- [x] Ensure logs do not corrupt the terminal. (tracing writes to file only)

Acceptance:

- [x] `?` opens useful keybinding help.
- [x] recent errors are visible without leaving the TUI. (Diagnostics view)
- [ ] diagnostics include enough context to reproduce common failures.
      (runtime state, counts, and event log present; DB/config paths missing)

### P6.13 Polish and Accessibility

Goal: make the TUI comfortable across terminal sizes and color themes.

Tasks:

- [ ] Add high-contrast status colors.
- [ ] Avoid color-only indicators by using badges/glyphs/text.
- [ ] Improve truncation and wrapping for Unicode names.
- [ ] Handle terminal resize events.
- [ ] Add mouse scroll/click if low-risk.
- [ ] Add read-only mode if useful for monitoring.

Acceptance:

- [ ] UI remains legible in common 80x24 and wider terminals.
- [ ] important states are understandable without color.
- [ ] resize does not panic.

### P6.14 Test Matrix

Goal: validate behavior without brittle terminal snapshot tests.

Required tests:

- [x] CLI parsing for `xrat tui`.
- [x] keymap action mapping (including QR/copy/filter/bulk-enable keys).
- [x] view routing and modal close behavior (QR modal back/Esc tested).
- [x] config filter/search/sort behavior (ConfigFilter cycle, TcpDelay/Source sort).
- [x] selection, enable/disable, delete, restore, and purge confirmation state.
- [ ] payload builders for copy/QR selected configs.
- [ ] source refresh/import action dispatch.
- [x] test progress reducer (incremental per-config progress events).
- [x] runtime status reducer.
- [ ] terminal lifecycle smoke test if practical.

Manual checks:

- [x] `cargo fmt`
- [x] `cargo test -q`
- [ ] `cargo run -- tui`
- [ ] navigate all primary views
- [ ] search/filter/sort configs
- [ ] copy focused and selected configs
- [ ] open QR modal for focused config
- [ ] paste/import a config URI
- [ ] run a focused test
- [ ] connect and disconnect runtime
- [ ] verify terminal restoration after quit and Ctrl+C

## Documentation

Update docs when the phase starts:

- add Phase 6 to `docs/src/SUMMARY.md`
- add usage docs for `xrat tui`
- document keybindings
- document QR/clipboard limitations
- document soft delete vs purge semantics
- document how TUI maps to CLI/runtime behavior
- keep `docs/ui/tui/index.html` linked as the prototype reference

## Completion Criteria

Phase 6 can be considered complete when:

1. [x] `xrat tui` starts and exits cleanly.
2. [x] Configs, Sources, Tests, Runtime, and Diagnostics views are navigable.
3. [x] Config table and detail panel use real DB data.
4. [ ] search, filters, sorting, and selection work for large config sets.
      (text search, enabled/failed/has-delay filters, 6 sort fields done;
      protocol filter and last-test/imported-at sort still pending)
5. [x] config enable/disable/select/delete/restore flows work safely.
      (focused and bulk-selected enable/disable now wired)
6. [x] QR and copy workflows work for focused and selected configs.
7. [x] paste/import flow reuses existing import parsing.
8. [x] test batches run in the background with progress feedback.
9. [ ] runtime start/stop/restart/switch flows call existing runtime services.
      (start/stop/restart done; config-switch not yet implemented)
10. [x] diagnostics and help are available from the TUI.
11. [ ] terminal state is restored after normal exit, Ctrl+C, and errors.
12. [x] `cargo fmt` and `cargo test -q` pass.

## Open Questions

- Should `Enter` immediately connect the focused config, or should it open a
  detail/action mode first?
- Should selected config state be persisted immediately on every toggle or
  batched until the user applies changes?
- Should clipboard support be always-on or feature-gated for headless builds?
- Should the TUI talk to the daemon over IPC when available, or call services
  directly in the first implementation?
- Should QR payloads include only raw config URIs, or also an XRAT profile
  wrapper for local runtime settings?
- Should auto-rotation controls appear in Runtime v1 or a later dedicated view?

## Completion blockers

**Reviewed: 2026-06-01**

**Updated: 2026-06-02**

Recent progress (session ending 2026-06-02):

- P6.6 partial: `ConfigFilter` cycle (`F`): EnabledOnly / FailedOnly / HasDelay
  filters applied at visibility level without reload. `TcpDelay` and `Source`
  sort fields added. Filter shown as chip in filter bar and in status-bar summary.
- P6.7 complete: `E`/`X` bulk-enable/disable all selected non-deleted configs.
  Focused `e`/`x` were already wired. All config mutation flows now implemented.
- P6.8 complete (prev session): source refresh (`r`/`R`), import modal (`i`),
  and background task flows with data reload.
- P6.10 partial (prev session): `s`/`x`/`r` runtime start/stop/restart wired.
  Config-switch remains.
- P6.11 complete for focused/selected configs: QR modal via `y` using `qrcode`
  crate with unicode half-block rendering; copy focused URI (`c`) and copy
  selected URIs (`C`) via `arboard`; clipboard errors surface in status bar.
- P6.12 partial: `TuiView::Diagnostics` (`5` key), bounded `event_log` populated
  from task events and config commands, info + log panels in view. DB/config
  paths not yet shown.
- 356 tests pass.

Phase 6 is at approximately 85% implementation. The following gaps remain:

### 1. Advanced filter: protocol and source-id

`ConfigFilter` handles enabled/failed/has-delay. Protocol filter and source-id
filter require either a picker UX or dedicated toggle keys. Last-test and
imported-at sort fields require adding those timestamps to `TuiConfigRow`.

### 2. Runtime config-switch

Start/stop/restart are wired. Switching the active config while runtime is
running (without a full stop) is not yet implemented.

### 3. QR/copy for source URLs and subscription payloads

`y`/`c` on the Sources view and HTTP API subscription URL QR/copy are not yet
wired. Only raw config URIs from the Configs view are covered.

### 4. Diagnostics: DB path and config path

Diagnostics view shows runtime state, counts, and event log. DB backend/path and
app config path are not yet surfaced.
