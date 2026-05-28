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

## Current Goal

Continue Phase 6 by making the Configs view useful for larger fleets:

1. add visible row window/table scrolling for long config lists
2. add search/filter/sort state and reducers
3. add selected-row toggling in the UI state
4. prepare config actions for enable/disable/select/delete/restore dispatch
5. keep the TUI responsive while data reloads

Progress estimate: **~15%** complete.

## Remaining Gaps

1. Add remaining Phase 6 crates when their slices begin:
   - `tui-qrcode`
   - input/modal helpers such as `tui-input` or `tui-textarea`
   - clipboard support, likely feature-gated
2. Expand `src/tui/` module tree for data loading, tasks, concrete views, and
   widgets.
3. Add panic/error-safe terminal cleanup beyond the current normal Drop-based
   cleanup.
4. Finish Configs interactions, then implement Sources, Tests, Runtime,
   Diagnostics, Help, QR, Paste, and confirmation views/modals.
5. Wire TUI actions to shared repository, import, tester, runtime, and daemon
   services instead of shelling out to CLI commands.
6. Add focused tests for reducers/state transitions, filters, sort, selection,
   confirmation state, payload builders, and task progress reducers.
7. Run broad verification in an environment that permits daemon sockets,
   ephemeral ports, and runtime process tests.

## Immediate Next Slice

1. Add table window/scroll state so long config lists keep the focused row
   visible.
2. Add filter/search model state for text, protocol, enabled, selected, failed,
   and deleted visibility.
3. Add sort model state for ID, name, protocol, source, real delay, TCP delay,
   and status.
4. Add in-memory selected toggle state as a stepping stone before DB-backed
   selection mutations.
5. Add tests for visible row sync, filter/search combinations, and sort order.

## Verification

- `cargo fmt` passed.
- `cargo test -q tui::` passed.
- `cargo test -q parses_tui_subcommand` passed.
- Full `cargo test -q` was attempted but blocked by sandbox/runtime restrictions
  unrelated to this slice: daemon socket reachability, ephemeral inbound port
  allocation, and local runtime process startup failed with permission/runtime
  errors.
