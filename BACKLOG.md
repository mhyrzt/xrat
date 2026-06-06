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

## 03. Medium-hard, P0: Migration checksum recovery must be future-proof

### Status

Done

### Problem

Migration 19 has triggered a SQLx checksum mismatch after this development
sequence:

- migration code and SQL files were written
- the app was tested manually, which applied the migration to the local database
- changes were committed
- formatting was run after the migration had already been applied
- running the app again failed with a migration checksum error for migration 19
- a repair was introduced, but it appears to be a temporary compatibility patch
  rather than a future-proof migration policy

### Possible root cause

SQLx stores the checksum of each applied migration in `_sqlx_migrations`.
Formatting or otherwise editing a migration file after any developer/user
database has applied it changes the embedded checksum for that migration. On the
next startup, SQLx compares the stored checksum against the current file and
rejects the migration history.

The current migration-19 repair likely updates a known legacy checksum to the
current checksum. That handles one exact historical mismatch but does not solve
the general process problem: future edits to already-applied migrations can
create new checksums that the repair code does not know about.

### Changes required

- Audit `src/db/schema.rs` migration repair logic and decide whether the
  migration-19 repair should remain, be narrowed, or be replaced with explicit
  operator guidance.
- Add a repository policy/check that prevents accidental edits to existing
  migration files once applied or released. Candidate options:
  - document and enforce "append-only migrations" in review/CI
  - add a checksum manifest for migrations and verify it in CI
  - add a local dev command that reports changed historical migrations
- Ensure migration error messages clearly distinguish checksum mismatch, dirty
  migration, and missing migration states.
- Document the recovery path for local development databases separately from
  released user databases.

### Verification

- Regression test for the known migration-19 checksum case, if the compatibility
  path remains.
- Test that checksum mismatch errors include the migration number and actionable
  guidance.
- CI or script verification that historical migration files were not modified
  unexpectedly.

## 04. Medium, P1: Polish rotate and proxy endpoint CLI output

### Status

Done

### Problem

Some human CLI messages are unfriendly or expose implementation details.

Examples:

```text
xrat rotate status
...
never triggered
```

The phrase "never triggered" is unclear to users. It should explain what has
not happened, for example that automatic rotation has not run yet.

```text
xrat proxy endpoints
Proxy endpoints
SOCKS5  socks5://0.0.0.0:18200 (host 172.17.70.159:18200)
```

The endpoint output shows both bind address and host address. For copy/paste
usage, showing `0.0.0.0` is noisy and less useful than showing the reachable
host IP directly.

### Possible root cause

The CLI is probably rendering raw runtime/bind state directly instead of
mapping it into user-facing endpoint values. `0.0.0.0` is correct as a bind
address, but it is not the best display address for clients. Likewise, rotate
status likely exposes internal scheduler state names instead of translating them
into human-readable status text.

### Changes required

- Update `xrat rotate status` human output so empty scheduler history reads as a
  clear phrase such as "not run yet" or "auto-rotation has not run yet".
- Update `xrat proxy endpoints` human output to show the reachable host address
  in the endpoint URI when the runtime binds to `0.0.0.0`.
- Keep JSON output stable unless there is a deliberate API/versioned change.
- Audit nearby CLI commands for similar raw implementation details in human
  output.
- Update docs examples under `docs/src/02-cli/` if output changes.

### Verification

- Add or update CLI output tests for rotate status empty-history text.
- Add or update endpoint formatting tests for `0.0.0.0` bind addresses.
- Manually verify `xrat proxy endpoints` against a running runtime.

## 05. Medium, P1: Standardize address/port columns and empty table cells

### Status

Done

### Problem

Tables that show a config address should split it into separate `Address` and
`Port` columns in both CLI and TUI views. Human table output should also avoid
blank cells; empty values should render as a consistent placeholder such as
`-`.

### Possible root cause

Config display code appears to predate the current table conventions and likely
passes combined address values or optional strings directly into renderers. That
makes port sorting/scanning harder and causes sparse rows to look broken when a
field is missing.

### Changes required

- Audit CLI tables that include config address data and split combined
  `Address` values into `Address` and `Port` columns.
- Audit TUI tables with config address data and make the same split.
- Add a shared display helper for human-table empty cells if one does not
  already exist.
- Preserve machine-readable formats unless an explicit schema change is chosen.
- Update docs examples that show table headers or rows.

### Verification

- CLI output tests for address/port table headers and empty-cell placeholders.
- TUI rendering test or snapshot-style check for config table columns.
- Manual check for `list`, `test`, scan-related output, and TUI config lists.
