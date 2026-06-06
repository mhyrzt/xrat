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

## 02. Easy, P2: Polish QR config modal

### Status

Planned

### Goal

Improve the QR modal in `src/tui/view/modals.rs` (`render_qr_modal`). The
current modal is too large, has insufficient padding from the border, and does
not look centered/aligned in the viewport.

### Changes required

- Rework modal sizing/centering. Current caller in `src/tui/view/mod.rs` uses
  `centered_rect(60, 80, area)`, which likely makes the QR modal oversized.
- Add inner padding so the QR code does not sit against the border.
- Use a specific modal title such as `Config QR`, `Source QR`, or `API QR`
  instead of generic `QR: ...`.
- Replace raw help text with a concise `[Esc] Close` hint styled consistently
  with the rest of the TUI.
- Do not show the full config/subscription URI as a separate truncated line.
- Show the config/subscription label under the QR code, centered.
- Keep behavior consistent for config QR, source QR, and API QR.

### UI notes

- Treat the QR modal as a compact utility dialog.
- Let the QR code define modal width, then add fixed horizontal padding.
- Use muted styling for the centered label under the QR code.
- Keep title and footer text short enough to avoid wrapping at narrow terminal
  widths.

### Nice to have

- Investigate whether a small center icon from `media/icons/` (for example
  `xrat-icon-192x192.png`) can be added without reducing QR scanner
  reliability.

### Verification

- Add or update modal rendering snapshot/unit coverage if existing TUI helpers
  support it.
- Manually verify at small and large terminal sizes that border, padding, label,
  and `[Esc] Close` do not overlap or clip.

## 03. Medium, P1: Add `xrat daemon restart`

### Status

Planned

### Goal

Give users a simple command to restart the daemon after changing `config.toml`.
Today `src/cli/daemon.rs` only supports `start`, `status`, `stop`, `install`,
and `uninstall`.

### Desired behavior

```text
xrat daemon restart
```

- Stop the running daemon, reload config, and start fresh.
- Reuse existing IPC and start/stop flows instead of duplicating daemon logic.
- Re-read `config.toml` as part of restart.
- Cleanly restart the active runtime session as part of daemon restart.
- If the daemon is not running, start it.
- Use the app IPC/start flow even when a systemd user-service is installed, so
  restart behavior stays consistent across manual and service-managed daemons.

### Changes required

- `src/cli/daemon.rs`: add `Restart(DaemonRestartArgs)`.
- `src/app/commands/daemon.rs`: add restart dispatch.
- Reuse `ipc::daemon_shutdown_daemon` and the existing start flow.
- Print restart progress/results with the same clean style as existing daemon
  commands; avoid raw/debug-looking output.
- `src/cli/tests/cases/runtime_parse/daemon.rs`: add parse coverage.
- Docs and generated man/completions: document the new subcommand.

### Verification

- CLI parse test for `xrat daemon restart`.
- Command-level test for stop-then-start behavior where existing test helpers
  allow it.
- Manual verification for:
  - daemon already running
  - daemon not running
  - systemd user-service installed; restart should still use the app IPC/start
    flow

### Decisions

- Use plain stop/start for the first version. A graceful in-place reload can be
  a later feature, but it is not required for a standard `restart` command.
- Always use the app IPC/start flow; do not delegate to systemd for the first
  version.

## 04. Medium, P2: TUI spinner during runtime switch

### Status

Planned

### Goal

Show a visible in-progress indicator while the TUI switches runtime configs.
Currently `RuntimeOp` appears as plain running text in
`src/tui/run/tasks/runtime.rs` and `src/tui/task/mod.rs`
(`TuiTaskKind::RuntimeOp`), which can read as frozen.

### Changes required

- `src/tui/task/`: track in-progress task kind and a tick/frame counter.
- `src/tui/view/`: render a spinner or animated marker next to runtime status,
  likely in `src/tui/view/configs/runtime.rs`.
- `src/tui/run/`: ensure the render loop ticks while a task is in flight so the
  spinner animates instead of waiting for input.
- Clear the spinner on task completion or failure and show the final state.

### Verification

- Rendering test for in-progress state.
- Rendering or state test showing spinner clears on completion.
- Manual TUI verification during a runtime switch.

### Decisions

- Show the spinner only for `RuntimeOp`.
- Use Unicode spinner frames, matching the existing rich TUI/tape assumptions.

## 05. Medium, P1: Review and finish the new `validate` command

### Status

Planned

### Goal

Settle the new untracked `validate` command before it becomes part of the stable
CLI surface.

Current files:

- `src/cli/validate.rs`
- `src/app/commands/validate.rs`

Current behavior:

```text
xrat validate <path>
```

It checks that a `config.toml` exists, parses, and is internally consistent.

### Review points

- Stage vocabulary drift:
  `validate_runtime` allows rotation `test_stages` of `icmp`, `ping`,
  `real_delay`, and `download`, while `ConnectionTestStage` only has `Icmp`,
  `RealDelay`, and `Download`. Confirm whether `ping` is a real alias or a
  phantom value, and whether `upload` or `tcp` should be validated.
- Testing/runtime mismatch:
  `validate_testing`, rotation stage lists, and `test_stage_name` do not appear
  to share one authoritative set of stage names.
- Hardcoded string matching:
  `engine`, stage names, and network values such as `tcp`/`udp` duplicate typed
  enum knowledge. Prefer parsing into existing domain enums when possible.
- Error aggregation:
  render validation failures with clean multi-line output through shared output
  helpers. Avoid ugly `; `-joined error blobs for human output.
- Secret resolution:
  validation should not require env/file secret values to exist for the default
  lint path. Validate secret references structurally and reserve actual
  resolution for runtime or an explicit future strict validation mode.

### Wiring to confirm

- Command is registered in `src/cli/mod.rs`.
- Handler is registered in `src/app/commands/mod.rs`.
- `xrat validate --help` output is correct.
- Human validation output is pretty and consistent with existing commands.
- Docs are added under `docs/src/02-cli/`.
- Generated man pages and completions include the command.

### Verification

- Unit tests for happy path and each validation error class.
- CLI parse test in `src/cli/tests/` if behavior can be validated without
  external services.
- Manual command examples:

```text
xrat validate config.toml
xrat validate --help
```

### Decisions

- Require an explicit path for the first version.
- Support `--format json` for machine-readable validation output.
- Keep invalid config exits non-zero through the existing `AppError` path.

## 06. Medium-hard, P0: TUI fails to recover from stale runtime PID after reboot

### Status

Planned

### Problem

After logging in to a machine that had been powered off, `xrat tui` showed
`daemon_restart_reattach_rejected_pid_missing`. The TUI did not restore the
previous runtime/config automatically; the config had to be selected manually.

### Investigation

- Reproduce the boot/login path with a persisted daemon/runtime session whose
  proxy process PID no longer exists.
- Confirm whether daemon restart/reattach treats a missing PID as a hard failure
  instead of falling back to the persisted runtime intent.
- Inspect how the failure is recorded and displayed so it can become an
  actionable recovery event instead of an opaque failure key.

### Likely changes

- When a persisted runtime session points to a missing PID after reboot, clear
  the stale process attachment.
- Reconnect or restart using the persisted config when possible.
- Record a readable event for stale PID recovery.

### Verification

- Regression test for restart/reattach with a missing PID and persisted config
  state.
- Manual TUI verification after simulating a stale persisted runtime session.

## 07. Medium-hard, P0: `xrat upgrade` migration failure on next run

### Status

Planned

### Problem

After running `xrat upgrade`, the next command invocation can fail with a
database migration error. The exact error text has not been captured yet.

### Current understanding

`xrat upgrade` swaps the binary through
`src/app/commands/upgrade/mod.rs` -> `install_binary`. It does not run database
migrations. Migrations run on the next command invocation through
`src/db/schema.rs` (`SQLITE_MIGRATOR.run` / `POSTGRES_MIGRATOR.run`, using
`sqlx::migrate!`). As a result, migration failures appear during the first
post-upgrade command instead of during the upgrade action.

The most likely cause is an sqlx migration checksum/version mismatch. sqlx
stores a checksum per applied migration in `_sqlx_migrations`; if a migration
file that was already shipped is edited later, the embedded checksum in the new
binary no longer matches the stored checksum and sqlx reports a version
mismatch. Other possibilities are a new migration that fails against existing
data or a partially applied/dirty migration row.

### Investigation

- Reproduce the failure and capture the exact error string.
- Audit `migrations/sqlite/` and `migrations/postgres/` history for any edited
  already-released migration files.
- Compare SQLite and Postgres migration trees for count/order drift.
- Check whether the failure is checksum mismatch, dirty migration state, or a
  forward migration that fails against real existing data.

### Likely changes

- Add a clear policy: never edit shipped migrations; always add a new ordered
  migration.
- Wrap migration errors in `src/db/schema.rs` with actionable context:
  migration version, likely cause, and recovery guidance.
- Consider running migrations as an explicit step during or immediately after
  `xrat upgrade`, so failures are associated with the upgrade.
- Consider adding `xrat db migrate` or a targeted repair path if dirty or
  mismatched migration states need user recovery.

### Verification

- Regression test: apply an old schema, run the current migrator, and assert it
  completes cleanly.
- Test error formatting for a migration failure path if the migrator can be
  isolated.
- Manually verify `xrat upgrade` failure messaging once the reproduction is
  known.

## 08. Hard, P1: Docker-style short refs for configs and subscriptions

### Status

Planned

### Goal

Allow user-facing prefix handles such as:

```text
xrat connect a1b2
```

This gives a Docker-like UX while keeping numeric database primary keys as
internal implementation details.

### Decision

Keep existing `BIGSERIAL` / `INTEGER` primary keys. Do not replace primary keys
with random strings, because that would force foreign keys to text, rewrite
child rows, increase index size, and lose monotonic insert order. Add a separate
stable user-facing `ref` column instead.

### Data model

- Add `ref TEXT NOT NULL UNIQUE` to `configs`.
- Add `ref TEXT NOT NULL UNIQUE` to `subscriptions`.
- Generate refs as random 12-character hex strings on insert.
- Refs are stable across edits.
- Do not use content hashes unless deduplication signal becomes a later goal.

Prefix lookup example:

```sql
SELECT id, ref
FROM configs
WHERE ref LIKE 'a1b2%'
  AND deleted_at IS NULL;
```

Resolution behavior:

- `0` rows: not found.
- `1` row: resolve to the internal id.
- More than `1` row: ambiguous; ask for more characters.

### Changes required

- `migrations/sqlite/` and `migrations/postgres/`: add ordered migrations for
  `ref` columns and unique indexes; backfill existing rows.
- `src/db/record/`: add `ref` to config and subscription records.
- `src/db/repository/`: generate refs on insert and add prefix-resolve helpers
  for configs and subscriptions.
- `src/db/database/`: ensure unique indexes are applied for both database
  backends.
- `src/cli/`: accept ref prefixes anywhere config/subscription ids are accepted:
  `connect`, `show config`, `show subscription`, `parse`, `test`,
  `delete config`, `delete subscription`, and similar commands.
- `src/app/commands/`: resolve ref prefixes to internal ids before calling
  existing command logic.
- `src/app/commands/output.rs`: display short refs in table/tsv/json/csv output
  where users currently need ids.
- Keep list/show/status output polished across table/tsv/json/csv formats;
  match existing column alignment, labels, and formatting conventions.
- `src/tui/`: show refs in list/detail views and update id-based adapters if
  needed.
- `src/server/`: expose refs in API responses while keeping existing numeric-id
  routes backward compatible. Add ref-prefix lookup only for routes where a
  config/subscription identifier is already user-supplied, with the same
  not-found and ambiguous-prefix behavior as the CLI.
- `src/support/`: add shared helpers for random ref generation and short-form
  display.
- `docs/src/02-cli/`: document ref usage and prefix matching.

### Verification

- Ref generation uniqueness tests.
- Prefix resolution tests for not-found, exact single match, and ambiguous
  match.
- SQLite and Postgres repository tests where existing helpers make both
  practical.
- CLI parse tests for ref arguments in `src/cli/tests/`.
- Output tests or snapshots for list/show/status formats.

### Decisions

- Accept both numeric ids and refs during the transition.
- Roll out refs to configs and subscriptions together.
- Show the first 8 characters by default.

## 09. Hard, P1: Rename rotation command and add proxy helpers

### Status

Planned

### Goal

Make proxy-related CLI names match user intent:

- `rotate`: automatic config rotation scheduling.
- `proxy`: local proxy endpoints and host/session integration.

Current `xrat proxy start|status|stop` controls rotation scheduling. The
`proxy` namespace should instead focus on local proxy URLs, shell proxy env
helpers, and Linux desktop proxy settings.

### Target command model

```text
xrat rotate start
xrat rotate stop
xrat rotate status

xrat proxy endpoints
xrat proxy endpoints --json

xrat proxy shell enable
xrat proxy shell disable
xrat proxy shell status

xrat proxy desktop enable
xrat proxy desktop disable
xrat proxy desktop status
```

Remove the old rotation commands from the `proxy` namespace when the `rotate`
command lands:

```text
xrat proxy start   removed
xrat proxy stop    removed
xrat proxy status  removed
```

### `xrat proxy endpoints`

Show active local proxy endpoints for quick use. Use `endpoints` instead of
`inbounds` because it is less engine-specific and describes the user-facing
URLs.

Example output:

```text
HTTP         http://127.0.0.1:18201
SOCKS5       socks5://127.0.0.1:18200
Shadowsocks  ss://...@192.168.1.20:18202
```

Requirements:

- Show only active runtime inbounds: HTTP, SOCKS5, and Shadowsocks.
- If the inbound bind host is `0.0.0.0`, display the machine LAN IP for easy
  local-network use.
- Otherwise display the configured bind host, normally `127.0.0.1`.
- Endpoint output must be clean and aligned like the rest of xrat command
  output, not raw debug formatting.
- Reuse or extend `format_inbound_endpoint` in
  `src/app/commands/runtime_output.rs`.

### `xrat proxy shell`

Use case: proxy only the current terminal session and its child processes,
without changing desktop or system proxy settings.

Requirements:

- `enable` prints shell-specific commands that set `http_proxy`, `https_proxy`,
  `all_proxy`, and uppercase variants.
- For `http_proxy` and `https_proxy`, prefer the active HTTP inbound when it is
  available; otherwise fall back to the active SOCKS inbound.
- For `all_proxy` / `ALL_PROXY`, prefer the active SOCKS inbound when it is
  available; otherwise fall back to the active HTTP inbound.
- If no usable HTTP or SOCKS inbound is active, return an error.
- `disable` prints shell-specific commands that unset those variables.
- `status` inspects the environment inherited by `xrat` and reports whether the
  current shell points at active xrat endpoints.
- Do not edit shell startup files such as `.bashrc`, `.zshrc`, fish config, or
  similar files.

Bash/zsh usage:

```sh
eval "$(xrat proxy shell enable)"
eval "$(xrat proxy shell disable)"
```

Fish usage:

```fish
xrat proxy shell enable | source
xrat proxy shell disable | source
```

Docs should also suggest optional user aliases/functions for convenience, while
making clear that xrat itself does not edit shell startup files. Example names:

```sh
alias xrat-proxy-on='eval "$(xrat proxy shell enable)"'
alias xrat-proxy-off='eval "$(xrat proxy shell disable)"'
```

For fish, document equivalent functions or abbreviations that pipe to `source`.

Shell detection:

- Detect from `$SHELL` first: `bash`, `zsh`, or `fish`.
- Fall back to the parent process name via `/proc/$PPID/comm`.
- Allow explicit override with `--shell bash|zsh|fish`.
- Document why bash/zsh need `eval "$( ... )"`: a child process cannot mutate
  its parent shell environment. Fish can source from a pipe.

### `xrat proxy desktop`

Linux-only desktop proxy integration. This changes desktop environment proxy
settings, not every process on the system.

Requirements:

- Auto-detect desktop from `$XDG_CURRENT_DESKTOP`, `$DESKTOP_SESSION`, and
  `$XDG_SESSION_TYPE`.
- Allow explicit override with `--desktop gnome|kde|xfce`.
- Support GNOME first through `gsettings`.
- Add KDE and XFCE only after testing their backend-specific behavior.
- If unsupported, fail clearly and suggest `xrat proxy shell enable` for
  terminal-only proxying.
- Use `desktop` rather than `system`, because Linux has no single universal
  system proxy authority.

### Changes required

- `src/cli/proxy.rs`: add endpoint/shell/desktop commands and remove old
  `start|status|stop` rotation commands from the proxy namespace.
- `src/cli/`: add a `rotate` command that owns rotation scheduling.
- `src/app/commands/proxy.rs`: dispatch new proxy helpers.
- `src/app/commands/`: add a rotate handler or move current rotation logic out
  of `proxy.rs`.
- `src/app/commands/runtime_output.rs`: extend endpoint formatting for
  HTTP/SOCKS5/Shadowsocks and `0.0.0.0` LAN IP display.
- Reuse shared CLI output helpers and existing format conventions for all proxy
  helper output.
- Add shell helper module for bash/zsh/fish enable/disable output and status
  inspection.
- Add desktop helper module for Linux desktop detection and GNOME `gsettings`
  backend.
- Docs, man pages, and completions: document new command names and the removal
  of old proxy rotation commands.

### Verification

- CLI parse tests for new commands, removed old proxy rotation commands,
  `--shell`, `--desktop`, and `--json`.
- Command tests for endpoint formatting.
- Command tests for shell output in bash, zsh, and fish modes.
- Command tests for shell status environment inspection.
- Command tests for unsupported desktop errors.

### Decisions

- Remove `xrat proxy start|status|stop` when `xrat rotate` lands; do not keep
  deprecated aliases.
- For `proxy shell enable`, use HTTP for `http_proxy`/`https_proxy` when HTTP is
  active, otherwise fall back to SOCKS. Use SOCKS for `ALL_PROXY` when SOCKS is
  active, otherwise fall back to HTTP. Error if no usable inbound exists.
- If Shadowsocks auth/material is unavailable from runtime status, omit the
  Shadowsocks endpoint and show a clear note instead of rendering a partial URL
  or failing the whole command.

## 10. Medium, P2: PAC support through the Axum API

### Status

Planned

### Goal

Expose a Proxy Auto-Config (PAC) file so browsers and desktop environments can
use xrat endpoints with per-destination routing rules instead of blunt global
proxy settings.

Target route:

```text
GET /proxy.pac
```

Target helper commands:

```text
xrat proxy pac url
xrat proxy pac print
```

Later desktop integration:

```text
xrat proxy desktop enable --pac
```

### Design

- Serve PAC from the existing Axum server in `src/server/`.
- Return `Content-Type: application/x-ns-proxy-autoconfig`.
- Generate PAC from active runtime HTTP/SOCKS endpoints.
- Do not include Shadowsocks credentials or other secrets in the PAC.
- Keep `/proxy.pac` unauthenticated by default. PAC consumers such as browsers
  and desktop proxy settings often cannot send custom auth headers, and putting
  API keys in PAC URLs is leaky.
- Treat the PAC route as a local helper endpoint, not a protected management API.
- Strongly prefer loopback binding for PAC use. If the server is bound to LAN,
  the PAC should still expose only non-secret local endpoint data.

Example PAC behavior:

```js
function FindProxyForURL(url, host) {
  if (isPlainHostName(host) || shExpMatch(host, "*.local")) {
    return "DIRECT";
  }

  return "SOCKS5 127.0.0.1:18200; DIRECT";
}
```

### Changes required

- `src/server/routes/`: add a PAC route and register it in
  `src/server/routes/mod.rs`.
- Add a PAC rendering helper that takes active HTTP/SOCKS endpoint data and
  emits deterministic PAC JavaScript.
- Reuse endpoint selection rules from `xrat proxy shell enable`:
  - prefer HTTP for HTTP/HTTPS proxy use when HTTP is active
  - prefer SOCKS for general proxy use when SOCKS is active
  - fall back between HTTP and SOCKS when only one is active
- `src/app/commands/proxy.rs`: add `proxy pac url` and `proxy pac print`.
- `src/cli/proxy.rs`: add `pac` subcommands.
- Keep `proxy pac url` and `proxy pac print` output clean, copy-paste friendly,
  and consistent with existing CLI output style.
- `xrat proxy desktop enable --pac`: configure desktop proxy settings to use the
  PAC URL once desktop integration exists.
- Docs/man/completions: document PAC URL, unauthenticated route behavior, and
  loopback recommendation.

### Verification

- Unit tests for PAC rendering:
  - HTTP only
  - SOCKS only
  - HTTP + SOCKS
  - local/private host bypass rules
- Server route test for `/proxy.pac`, including content type.
- CLI parse tests for `proxy pac url` and `proxy pac print`.
- Manual browser/desktop test once `proxy desktop --pac` exists.

### Decisions

- Use the existing Axum API server for PAC.
- Keep `/proxy.pac` unauthenticated by default.
- Do not include Shadowsocks credentials in PAC output.
- Return a PAC that falls back to `DIRECT` after xrat proxies.

## 11. Hard, P2: TUI logs card with events, proxy logs, and stats tabs

### Status

Planned

### Goal

Replace the current config log view with a tabbed logs card. Current
`src/tui/view/configs/log.rs` only renders config `failure_reason` lines.

### Target tabs

1. `xrat events`: internal app events such as session changes, tests, rotation,
   health, daemon activity, and runtime transitions. Source:
   `src/app/events.rs` and `src/db/repository/events.rs`, the same data used by
   `xrat logs`.
2. `proxy engine`: raw proxy process logs from xray or sing-box. Source:
   process stdout/stderr or log files from `src/xray/process_mgmt/` and the
   sing-box equivalent.
3. `stats`: totals and live traffic data:
   - total download
   - total upload
   - current delay/ping
   - failed request count
   - live throughput graph

Stats source should be xray StatsService (`grpc`/`StatsService`) or the
sing-box Clash API. Feed the TUI through a poller and ring buffer; render with a
ratatui sparkline/chart widget.

### Cross-cutting UI requirements

- Render logs and events with aligned columns, readable timestamps, and
  level/kind coloring through the theme.
- Add reset/clear shortcuts per tab where useful:
  - clear visible log buffer
  - reset stat counters, if supported
- Wire shortcuts through `src/tui/keymap/` and `chord.rs`.
- Show shortcuts in the existing help/chrome UI.

### Changes required

- `src/tui/view/configs/log.rs`: rewrite as tab container plus per-tab renderers.
- `src/tui/app/types.rs` and related `src/tui/app/` modules: add active-tab
  state, stats ring buffer, and proxy-log buffer.
- `src/tui/data/`: add adapters for xrat events, proxy log tailing, and stats
  polling.
- `src/tui/keymap/` and `chord.rs`: add tab-switch keys, reset/clear shortcuts,
  and help entries.
- `src/xray/process_mgmt/` and sing-box runtime code: expose proxy log stream or
  log path and stats API clients if missing.
- Runtime config generation: ensure stats APIs are enabled when needed. Check
  `src/xray/parsing/core/api.rs` and `policy.rs`.

### Verification

- State tests for tab switching.
- Keymap tests for tab and reset/clear bindings.
- Stats buffer rollover tests.
- Manual TUI verification with active runtime, proxy logs, and stats enabled.

### Decisions

- Use one engine-neutral stats trait for now.
- Reset stats per runtime session.
- Use live tail for proxy logs.
