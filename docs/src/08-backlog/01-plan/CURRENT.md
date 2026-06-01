# Current Work: Phase 6 TUI Application

## Context

Phases 1 through 5 provide the local and API workflow: import, persistence,
testing, managed runtime, daemon ownership, auto-rotation, and an Axum HTTP API
for configs and subscription-compatible output.

Current planning work is focused on Phase 6: a Ratatui-based terminal UI that
turns those existing services into an interactive operations console.

The TUI should support the main day-to-day XRAT workflow:

- browse, filter, sort, select, enable, disable, delete, restore, and activate
  configs
- inspect subscription sources and refresh/import data
- run focused or batch connection tests with live progress
- start, stop, restart, and inspect the managed Xray runtime
- copy, paste, and show QR codes for config URIs, subscription URLs, selected
  config bundles, and local profile payloads
- surface diagnostics and recent operation failures without leaving the TUI

The UX/layout reference for this phase is `docs/ui/tui/index.html`.

## What Landed So Far

- Phase 6 detailed backlog added at `docs/src/08-backlog/01-plan/PHASE_6.md`.
- mdBook summary now includes the Phase 6 page.
- Roadmap Phase 6 section in `README.md` now links to the detailed plan.
- The Phase 6 plan explicitly references `docs/ui/tui/index.html` as the Ratatui
  UX prototype.
- Phase 6 scope is broken into implementation slices:
  - TUI entry and terminal lifecycle
  - app model, events, and keymap
  - layout chrome
  - config data loading
  - configs view rendering
  - search, filter, and sort
  - config actions
  - sources view
  - testing view and progress
  - runtime view
  - QR, clipboard, and paste modals
  - diagnostics and help
  - polish/accessibility
  - test matrix
- Required and recommended UI/UX crates are documented:
  - existing: `ratatui`, `tokio`, `tracing`
  - recommended: `crossterm`, `tui-qrcode`, `qrcode`, `arboard` or
    `cli-clipboard`, `unicode-width`, `textwrap`, `tui-input`, `tui-textarea`,
    and `strum`
  - optional ecosystem evaluation: `tui-widgets`
- QR workflows are included as first-class Phase 6 scope using `tui-qrcode`
  where practical.
- Soft delete, restore, and explicit purge semantics are carried forward into
  the TUI config-management plan.
- Initial Phase 6 vertical slice started:
  - `crossterm` added as the terminal event/backend dependency.
  - `xrat tui` CLI command added.
  - TUI app command handler added under `src/app/commands/tui.rs`.
  - `src/tui/` module tree started with app state, keymap, theme, renderer, and
    terminal runner.
  - placeholder Ratatui shell renders top status bar, mode rail, active content
    panel, footer key bar, and help modal.
  - global keys currently handled: `1`-`4`, `?`, `Esc`, `q`, Ctrl+C, `j/k`,
    arrows, and `/`.
  - terminal lifecycle enters raw mode and alternate screen, then restores raw
    mode, alternate screen, and cursor on exit.
  - focused tests added for `xrat tui` CLI parsing, key mapping, and app state
    transitions.
- Configs view data/render slice started:
  - TUI data model added for config rows and aggregate counts.
  - initial data loader uses existing config-with-latest-test repository helpers
    and hides deleted configs by default.
  - configs are initially sorted by real-delay, with untested configs last.
  - top status bar now shows total, enabled, selected, and failed counts.
  - Configs view now renders a real table with ID, name, protocol, address:port,
    network/security, delay, and status columns.
  - focused config detail panel shows endpoint, network, latest delay/TCP,
    source, status, actions, and latest failure reason.
  - row movement updates focused config state and clamps at list bounds.
  - focused tests added for count summaries, row formatting, and focus movement.
- Configs view interaction slice landed:
  - `/` edits an inline text search and the header shows active search/sort
    state.
  - `s` cycles sort modes for visible configs.
  - `f` toggles deleted-row visibility.
  - `Space`, `e`, `x`, `d`, `D`, and `r` dispatch select, enable, disable,
    soft-delete, purge, and restore actions through existing repository methods.
  - soft delete and purge use confirmation modals before mutating data.
  - rows reload after mutations and focused tests cover key mappings, reducers,
    confirmation state, and command dispatch.
- Sources view read-only slice landed:
  - TUI data loading now includes subscription sources from
    `list_subscriptions()`.
  - Sources view renders a real source table with ID, name, kind, config count,
    and update time.
  - focused source detail shows kind, value, counts, and timestamps.
  - `j/k` and arrow navigation move source focus independently from config
    focus.
- Runtime view read-only slice landed:
  - TUI data loading now includes current runtime/session status through
    `RuntimeService::status()`.
  - Runtime view renders status cards plus session, inbound, failure,
    transition, and database details.
  - start/stop/restart/log actions remain visible as coming-next actions until
    background task infrastructure lands.
- Tests view read-only slice landed:
  - TUI data loading now includes the latest connection-test run and recent
    result rows through existing database helpers.
  - Tests view renders scope, mode, latest run, queue/concurrency, progress
    counts, untested/failed summaries, and recent result rows.
  - test scope state exists for focused, selected, filtered, all enabled,
    failed, and stale/untested sets.
- Background task scaffold landed:
  - typed TUI task events and lifecycle state now exist under `src/tui/task.rs`.
  - the run loop drains task events without blocking terminal input.
  - deleted-filter data reload now uses the background task channel.
  - the status bar shows task state and reducer tests cover task completion.
- Tests background execution slice landed:
  - `s` in the Tests view starts a scoped background test batch.
  - the TUI reuses the existing connection-test executor and records persisted
    test results without shelling out to CLI text output.
  - completed test tasks reload DB-backed config rows and latest-test summaries.
  - cancellation is mapped in the keymap but still needs a cancellation-token
    path before it can stop in-flight tests.

## Current Goal

Continue Phase 6 by finishing background test control, then move to the next
service-specific task path:

1. add cancellation support for running test batches
2. improve test progress events beyond started/completed/failed summaries
3. add source refresh/import or runtime start/stop as the next background task
4. keep completion/failure visible in the relevant view and diagnostics buffer

Progress estimate: **~55-60%** complete.

## Remaining Gaps

1. Add remaining Phase 6 crates when their slices begin:
   - `tui-qrcode`
   - input/modal helpers such as `tui-input` or `tui-textarea`
   - clipboard support, likely feature-gated
2. Expand `src/tui/` module tree for data loading, tasks, concrete views, and
   widgets.
3. Add panic/error-safe terminal cleanup beyond the current normal Drop-based
   cleanup.
4. Finish remaining Configs polish, then implement Runtime actions, Diagnostics,
   QR, Paste, and the remaining service-specific background task flows.
5. Wire TUI actions to shared repository, import, tester, runtime, and daemon
   services instead of shelling out to CLI commands.
6. Add focused tests for reducers/state transitions, filters, sort, selection,
   confirmation state, payload builders, and task progress reducers.
7. Run broad verification in an environment that permits daemon sockets,
   ephemeral ports, and runtime process tests.

## Immediate Next Slice

1. Add cancellation-token support for test batches.
2. Emit incremental progress events while tests run.
3. Add source refresh/import or runtime operation dispatch through the same task
   channel.
4. Add reducer tests for the operation-specific task state.
5. Keep service-specific execution small and routed through existing app
   services.

## Verification

- `cargo fmt` passed.
- `cargo test -q tui::` passed.
- Full `cargo test -q` passed after the background test batch slice.

## Completion blockers

**Reviewed: 2026-06-01** **Resolved: 2026-06-01**

This is a living work tracker for Phase 6, not a completable backlog item. It
should be updated as Phase 6 progresses. The following factual inaccuracies were
found during review and have been resolved:

1. **Incorrect Phase 2.5 completion claim** - Resolved: Lifecycle commands
   (`select`, `enable`, `disable`, `delete`, `restore`) and `show` have been
   added to the CLI.

2. **Incorrect Phase 3 completion claim** - Resolved: `reqwest` `socks` feature
   added to `Cargo.toml`, and `Proxy::all(...).unwrap()` replaced with proper
   error handling in the real-delay prober.

3. **Progress estimate** - The "~15%" estimate may undercount current progress.
   The Configs view has a functional table with 7 columns, detail panel, focus
   navigation, aggregate counts, and real DB data loading. A more accurate
   estimate might be 25-35%.
