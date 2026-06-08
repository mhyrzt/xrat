# Hard, P2: TUI logs card with events, proxy logs, and stats tabs

### Status

In progress

### Progress

- Implemented `xrat events` and `proxy engine` tabs in the TUI logs card.
- Parsed xray engine log lines into structured time/level/feed/source/message
  columns and normalized feed labels to the engine name; unparseable lines stay
  raw.
- Added direct number-key log tab selection (`1`/`2`/`3`) alongside the existing
  `[`/`]` cycle, plus a placeholder `stats` tab.
- Added persisted-event clearing: `xrat logs clear [--yes]` (CLI) and the TUI
  `C p` clear chord, both deleting `events` rows after a confirm. Labeled
  "events (db)" to keep it distinct from the planned view-only buffer clears.

### Remaining plan

The remaining heavy pieces (live stats, API tab, view-only clears, and log
severity polish) are sequenced as focused commits A–F.

- **A — runtime config emits stats plumbing for both engines.** Add optional
  `api`/`stats`/`policy`/`routing` fields to `XrayConfig`
  (`src/xray/config/types.rs`), all
  `#[serde(skip_serializing_if = "Option::is_none")]` so probe configs and
  stats-disabled runtime serialize identically. Add
  `StatsSettings { enabled, host, port }` (default enabled, `127.0.0.1:10085`)
  to `RuntimeSettings` (`src/app/config/proxy/types.rs`). When enabled,
  `src/app/runtime_service/launch.rs` appends a `dokodemo-door` inbound tagged
  `api` and populates the new config fields; readiness inbound selection is
  unchanged. For managed sing-box sessions, pass `SingboxClashApi` into
  `generate_singbox_runtime_config` so `experimental.clash_api` binds the same
  localhost controller endpoint.
- **B — engine-neutral stats sources.** xray StatsService is gRPC-only. Add
  `tonic` + `prost` deps (rustls only, no `build.rs`/`protoc`); hand-write the
  prost messages (`QueryStatsRequest`, `Stat`, `QueryStatsResponse`) for
  `/xray.app.stats.command.StatsService/QueryStats`. New `src/xray/stats/` with
  `XrayStatsClient` + stat-name parsing. Define
  `trait StatsSource { async fn sample(&self) -> Result<StatsSample> }`
  (async-trait); `XrayStatsSource` for `xray`/`v2ray`, plus a sing-box source
  that samples the Clash API traffic endpoint exposed by
  `experimental.clash_api`. `StatsSample { at, uplink_total, downlink_total }`.
- **C — TUI stats poller, ring buffer, sparkline.** `src/tui/data/stats.rs`
  bounded `VecDeque` ring buffer (~120 points) reset on session id change; app
  state + new `stats_tx/rx` channel and ~1s interval gate in `src/tui/run/`;
  `spawn_poll_stats` polls only when running + stats enabled. Replace the
  `stats_lines` placeholder with header rows (total ↓/↑, throughput, delay,
  failed count) plus a `ratatui` `Sparkline`/`Chart` (confirm import in 0.30).
- **D — API tab via request-logging middleware.** Add `SOURCE_API` to
  `src/app/events.rs`; axum `middleware::from_fn_with_state` records each
  request as a fire-and-forget event (`source=api`, `kind=<method>`,
  `message="<METHOD> <path> -> <status>"`) wired in `build_router`. Add
  `TuiLogTab::Api` to `ORDER` (tabs become events/engine/api/stats, number keys
  `1`–`4`); `api_lines` filters events to `source == "api"`.
- **E — view-only clears (`C l`, `C s`).** Extend the `C` chord to
  `[("l","log view"), ("s","stats view"), ("p","events (db)")]`; add actions
  `ClearLogView`/`ClearStatsView`. Track per-tab view watermarks (events:
  `events_clear_before_id`; proxy: last-visible-row signature) so the periodic
  reload does not resurrect cleared rows; `ClearStatsView` empties the ring
  buffer. Update help modal + docs to distinguish view clears from the DB clear.
- **F — docs + backlog.** Update `docs/src/02-cli/tui.md` (stats/API tabs, full
  clear set, severity colors) and the `[runtime.stats]` config reference,
  including xray StatsService and sing-box Clash API behavior; mark this item
  Done.

Constraints: stats config is backward-compatible via `#[serde(default)]`; the
default-on xray api inbound and sing-box Clash API controller each bind an extra
localhost port (gated by config); stats RPC/API failures must stay silent in the
TUI; `tonic`/`prost` must build on the musl release target (rustls only, no
`build.rs`). Verify a `--locked` build after adding deps.

### Goal

Replace the current config log view with a tabbed logs card. Current
`src/tui/view/configs/log.rs` only renders config `failure_reason` lines.

### Target tabs

1. `XRAT`: internal app events such as session changes, tests, rotation, health,
   daemon activity, and runtime transitions. Source: `src/app/events.rs` and
   `src/db/repository/events.rs`, the same data used by `xrat logs`.
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

### Proxy log **refinements**

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
- Use a consistent severity color schema across app events and proxy logs:
  critical/fatal/panic/error in red, warn/warning in yellow, info/debug/trace in
  neutral or accent colors. When a proxy line is unparseable, infer severity
  from known keywords and stderr context without hiding the raw message.
- Add reset/clear shortcuts per tab where useful:
  - clear visible log buffer
  - clear visible stats buffer/counters in the TUI only
  - clear persisted logs/stats from the database after an explicit confirm step
  - expose equivalent clear behavior for `xrat logs` where it owns persisted
    event/log records
  - keep view-only clears separate from database clears in shortcut labels, help
    text, and command output
  - record database clear actions as best-effort events when possible without
    failing the primary clear operation
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
- `src/app/commands/logs.rs` and related CLI parsing: add clear/reset support
  for persisted `xrat logs` data, using explicit command naming and confirmation
  consistent with destructive operations elsewhere in the CLI.
- `src/xray/process_mgmt/` and sing-box runtime code: expose proxy log stream or
  log path and stats API clients if missing.
- Proxy log rendering: normalize engine feed labels and parse xray log lines
  into structured fields where practical.
- Runtime config generation: ensure stats APIs are enabled when needed. Check
  `src/xray/parsing/core/api.rs` and `policy.rs`.

### Verification

- State tests for tab switching.
- Keymap tests for tab and reset/clear bindings.
- TUI state tests proving view-only clear does not delete persisted records.
- CLI tests for `xrat logs` clear/reset parsing and confirmation behavior.
- Parser tests for representative xray log lines.
- Stats buffer rollover tests.
- Manual TUI verification with active runtime, proxy logs, active API server,
  and stats enabled.

### Decisions

- Use one engine-neutral stats trait for now.
- Reset stats per runtime session.
- Use live tail for proxy logs.
