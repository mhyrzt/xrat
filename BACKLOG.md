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

---

## 02. Hard, P2: TUI logs card with events, proxy logs, and stats tabs

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

The remaining heavy pieces (live stats, API tab, view-only clears) are sequenced
as focused commits A–F.

- **A — xray runtime config emits stats plumbing.** Add optional
  `api`/`stats`/`policy`/`routing` fields to `XrayConfig`
  (`src/xray/config/types.rs`), all
  `#[serde(skip_serializing_if = "Option::is_none")]` so probe configs and
  stats-disabled runtime serialize identically. Add
  `StatsSettings { enabled, host, port }` (default enabled, `127.0.0.1:10085`)
  to `RuntimeSettings` (`src/app/config/proxy/types.rs`). When enabled,
  `src/app/runtime_service/launch.rs` appends a `dokodemo-door` inbound tagged
  `api` and populates the new config fields; readiness inbound selection is
  unchanged.
- **B — gRPC StatsService client + engine-neutral trait.** xray StatsService is
  gRPC-only. Add `tonic` + `prost` deps (rustls only, no `build.rs`/`protoc`);
  hand-write the prost messages (`QueryStatsRequest`, `Stat`,
  `QueryStatsResponse`) for `/xray.app.stats.command.StatsService/QueryStats`.
  New `src/xray/stats/` with `XrayStatsClient` + stat-name parsing. Define
  `trait StatsSource { async fn sample(&self) -> Result<StatsSample> }`
  (async-trait); `XrayStatsSource` for `xray`/`v2ray`, sing-box returns
  unsupported for now. `StatsSample { at, uplink_total, downlink_total }`.
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
  clear set) and the `[runtime.stats]` config reference; mark this item Done.

Constraints: stats config is backward-compatible via `#[serde(default)]`; the
default-on api inbound binds an extra localhost port (gated by config); stats
RPC failures must stay silent in the TUI; `tonic`/`prost` must build on the musl
release target (rustls only, no `build.rs`). Verify a `--locked` build after
adding deps.

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

---

## 03. Hard, P1: Proper authentication for a management HTTP API

### Status

Planned

### Goal

Add real authentication suitable for state-changing endpoints before exposing a
management API (add/delete/edit configs, connect/disconnect, rotation control,
etc.). The current `?key=KEY` scheme (`src/server/auth.rs`,
`src/server/routes/*`) is acceptable for read-only, non-critical endpoints but
must not gate mutations.

### Current behavior

- API key passed as a `?key=` query parameter on each route
  (`ConfigsQuery`/`JsonQuery` `key` field), validated by `require_api_key` in
  `src/server/auth.rs`.
- All routes are read-only `GET` (`src/server/routes/mod.rs`): health, json,
  b64, configs list/get, proxy.pac.
- Single shared key compared with `provided != expected`.

### Weaknesses to address for write/management endpoints

- Key in the query string leaks into server access logs, browser history,
  bookmarks, and `Referer` headers — unsafe for privileged operations.
- Comparison is not constant-time (`!=`), a timing side channel.
- No header-based credential (`Authorization: Bearer`/`X-API-Key`).
- No separation of capability: one key grants everything; no read-only vs manage
  distinction.
- No rate limiting, lockout, or audit trail for failed/privileged calls.

### Changes required

- Introduce header-based auth (`Authorization: Bearer <token>` and/ordo i
  `X-API-Key`) for any non-`GET`/management route; keep `?key=` allowed only for
  the existing read-only endpoints (or behind a compat flag) and document it as
  low-sensitivity.
- Use a constant-time comparison for secrets.
- Model capabilities/scopes: at minimum read-only vs manage, so a read token
  cannot mutate state. Consider per-scope tokens.
- Apply auth via middleware on the management router so new routes are protected
  by default (ties into the API request-logging middleware from item 02 — record
  auth failures as best-effort events).
- Bind management endpoints to localhost by default and require explicit opt-in
  to expose them; document the exposure model.
- Add rate limiting / failed-attempt backoff for auth on management routes.
- Record privileged actions as audit events (`src/app/events.rs`).
- CLI/config: generate/store management tokens; surface them through config the
  same way the existing `api_key` is managed.

### Verification

- Auth unit tests: header token accepted, query key rejected on management
  routes, read scope cannot mutate, constant-time path exercised.
- Middleware tests that management routes 401/403 without a valid token and that
  failures record audit events.
- Manual: exercise a sample management endpoint with/without a valid token over
  localhost.

### Decisions

- `?key=` stays only for read-only, non-critical endpoints. Management endpoints
  require header tokens with scopes, constant-time comparison, and
  localhost-only binding by default.

- also change `?key=` to `?token=`

====Unstructured Todos================

2. when in tui and running test (t <Key>) is it possible to show the results of
   test for a finshied config immedietly after it's done
3. `xrat proxy endpoints` -> `xrat proxy info` or `xrat proxy show` whatever you
   think is better (also change Proxy Endpoints outputs to sth better)
4. adding xrat proxy toggle -> automatically captures env vars and outputs new
   varaible
