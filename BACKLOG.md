# Backlog

This backlog is for future implementation planning. Each item should be
implemented as a focused change with its own tests and documentation updates
when user-facing behavior changes.

Difficulty guide:

- **Easy**: mostly docs, assets, isolated UI rendering, or small command wiring.
- **Medium**: focused code change with clear ownership and moderate tests.
- **Medium-hard**: bug or behavior change that needs reproduction and careful
  lifecycle/state handling.
- **Hard**: cross-layer feature touching CLI, app logic, persistence, runtime,
  server, TUI, or engine integrations.

Severity guide:

- **P0**: user-visible bug or migration/runtime recovery risk.
- **P1**: CLI or data-model change that affects common workflows.
- **P2**: TUI, docs, demos, and polish work.

Status guide:

- **Planned**: ready for implementation planning, not started.
- **In progress**: implementation started but not complete.
- **Blocked**: waiting on missing information or external dependency.
- **Done**: implemented and verified.

Implementation guide:

- For CLI-facing changes, output must be polished and consistent with the rest
  of xrat. Reuse shared output helpers and existing `--format` conventions
  instead of adding ugly ad hoc prints.
- Make focused commits frequently while implementing backlog items so progress
  is easy to review, test, and revert if needed.

## 01. Easy, P2: VHS tape demos in READMEs

### Status

Planned

### Goal

Replace the static `media/screenshot.png` preview with animated VHS demos.

### Current assets

- Tape files:
  - `media/tapes/tui.tape`
  - `media/tapes/cli.tape`
- Expected generated files:
  - `media/gif/tui.gif`
  - `media/gif/cli.gif`

Render commands:

```sh
mkdir -p media/gif
vhs media/tapes/tui.tape
vhs media/tapes/cli.tape
```

### Changes required

- Generate and commit `media/gif/tui.gif` and `media/gif/cli.gif`.
- Add a `Justfile` recipe, for example `just gifs`, that creates `media/gif/`
  and renders both VHS tapes.
- Replace the screenshot image block in `README.md`.
- Replace the screenshot image block in `docs/src/README.md`.
- Verify the mdBook asset path. `docs/src/README.md` currently uses a
  `media/...` path prefix, so the gifs may need to be copied under
  `docs/src/media/gif/` or referenced differently.

### Verification

- Confirm both gifs render locally.
- Confirm `just gifs` renders both gifs into `media/gif/`.
- Confirm README image paths resolve.
- Confirm mdBook renders the docs preview correctly.

### Decisions

- Use two separate gifs: one for CLI and one for TUI.
- Remove `media/screenshot.png` after the gifs are added.
- Keep gif regeneration manual through `just gifs`; do not add CI regeneration
  for the first pass.

## 02. Hard, P2: TUI logs card with events, proxy logs, and stats tabs

### Status

In progress

### Progress

- Implemented `xrat events` and `proxy engine` tabs in the TUI logs card.
- Left the `stats` tab and stats API/poller work for a later focused change.

### Goal

Replace the current config log view with a tabbed logs card. Current
`src/tui/view/configs/log.rs` only renders config `failure_reason` lines.

### Target tabs

1. `XRAT`: internal app events such as session changes, tests, rotation,
   health, daemon activity, and runtime transitions. Source: `src/app/events.rs`
   and `src/db/repository/events.rs`, the same data used by `xrat logs`.
2. `Engine`: raw proxy process logs from xray or sing-box. Source: process
   stdout/stderr or log files from `src/xray/process_mgmt/` and the sing-box
   equivalent.
3. `API`: Axum/web-server logs and HTTP API events, if the server path exposes
   enough structured data.
4. `stats`: totals and live traffic data:
   - total download
   - total upload
   - current delay/ping
   - failed request count
   - live throughput graph

Stats source should be xray StatsService (`grpc`/`StatsService`) or the sing-box
Clash API. Feed the TUI through a poller and ring buffer; render with a ratatui
sparkline/chart widget.

### Proxy log refinements

The first TUI logs implementation likely exposed raw process stream labels and
raw engine lines to get observability working quickly. That is useful for
debugging, but the user-facing view should be more structured.

Current rough output:

```text
FEED          MESSAGE
xray out      2026/06/06 15:10:29.760343 [Warning] core: Xray 26.3.27 started
```

Desired behavior:

- Render the feed/source as `xray` by default instead of `xray out`.
- Include stdout/stderr only when it materially helps diagnose the issue.
- Parse known xray log lines into structured fields:
  - timestamp
  - level
  - component/source
  - message
- Keep unparseable lines visible as raw messages with sensible fallback fields.
- Consider sharing the structured log row model with `xrat logs` output where
  the ownership boundaries make sense.

### Cross-cutting UI requirements

- Render logs and events with aligned columns, readable timestamps, and
  level/kind coloring through the theme.
- Add reset/clear shortcuts per tab where useful:
  - clear visible log buffer
  - reset stat counters, if supported
- Wire shortcuts through `src/tui/keymap/` and `chord.rs`.
- Show shortcuts in the existing help/chrome UI.

### Changes required

- `src/tui/view/configs/log.rs`: rewrite as tab container plus per-tab
  renderers.
- `src/tui/app/types.rs` and related `src/tui/app/` modules: add active-tab
  state, stats ring buffer, and proxy-log buffer.
- `src/tui/data/`: add adapters for xrat events, proxy log tailing, and stats
  polling.
- `src/tui/keymap/` and `chord.rs`: add tab-switch keys, reset/clear shortcuts,
  and help entries.
- `src/xray/process_mgmt/` and sing-box runtime code: expose proxy log stream or
  log path and stats API clients if missing.
- Proxy log rendering: normalize engine feed labels and parse xray log lines
  into structured fields where practical.
- Runtime config generation: ensure stats APIs are enabled when needed. Check
  `src/xray/parsing/core/api.rs` and `policy.rs`.

### Verification

- State tests for tab switching.
- Keymap tests for tab and reset/clear bindings.
- Parser tests for representative xray log lines.
- Stats buffer rollover tests.
- Manual TUI verification with active runtime, proxy logs, active API server,
  and stats enabled.

### Decisions

- Use one engine-neutral stats trait for now.
- Reset stats per runtime session.
- Use live tail for proxy logs.
