# tui

Start the interactive terminal UI.

```bash
xrat tui
```

When installed via `install.sh` or `just install`, xrat also installs an
`xratui` shortcut next to the `xrat` binary:

```bash
xratui
```

The TUI has no command-specific flags. It uses the same global flags as other
commands, including `--database`, `--config`, `--xray`, `--v2ray`, `--sing-box`,
`-v`, and `-q`.

The TUI is an interactive view over xrat's shared database, subscription,
testing, and runtime services. It does not keep a separate copy of business
logic: config changes, imports, tests, and runtime operations use the same app
services as the CLI commands.

## Tabs

The TUI is a single dashboard. The top-left table has two tabs; switching the
tab also swaps the detail panel on the right. The Testing strip, the Logs panel,
and the Runtime panel stay visible under both tabs.

| Tab           | Purpose                                                                 |
| ------------- | ----------------------------------------------------------------------- |
| Configs       | Browse, filter, start, test, enable, disable, delete, and share configs |
| Subscriptions | Inspect subscriptions, refresh them, and share subscription/API URLs    |

Use `[` and `]` to move to the previous / next tab.

The TUI opens on the Configs tab. The bottom bar shows the version (with an
upgrade hint when a newer release is available) and a help shortcut. Test
batches are started and monitored from the Configs tab itself; there is no
separate Tests view.

## Global Keys

| Key           | Action                                            |
| ------------- | ------------------------------------------------- |
| `[`, `]`      | Switch to previous / next table tab               |
| `Tab`         | Cycle card focus (Table → Detail → Log → Runtime) |
| `Shift+Tab`   | Cycle card focus in reverse                       |
| `1`           | Focus the table card                              |
| `2`           | Focus the logs/events card                        |
| `3`           | Focus the detail card                             |
| `4`           | Focus the runtime card                            |
| `j`, `k`      | Move row / scroll the focused card down/up        |
| arrow keys    | Move row / scroll the focused card down/up        |
| `PgUp`, `PgDn`| Page the focused card up / down                   |
| `Home`, `End` | Jump to the top / bottom of the focused card      |
| `?`           | Open help                                         |
| `Esc`         | Close modal, leave search, or go back             |
| `q`, `Ctrl+C` | Quit                                              |

## Cards and Scrolling

The dashboard has four cards: the table (Configs/Subscriptions), Logs, the
detail panel, and Runtime. Card titles show their direct focus shortcuts (`1:`,
`2:`, `3:`, `4:`). `Tab` / `Shift+Tab` move focus between them; the focused card
is drawn with an accent border. `j`/`k` (or the arrow keys) move the row
selection when the table is focused, and scroll the focused card otherwise.
`PgUp`/`PgDn` step by a screenful and `Home`/`End` jump to the first/last row
or top/bottom of the card. Cards that overflow their height show a scrollbar.

In the Logs card, long messages wrap inside the message column and continuation
lines stay indented under it, so the time, level, source, and kind columns stay
aligned and one entry never blends into the next.

## Configs Tab

The Configs tab shows stored configs with latest test summaries and config
state. The status marker column uses `●` for active, `✕` for soft-deleted, `○`
for disabled, and `!` for failed configs. Long names are truncated in the table.
It supports focused actions, test batches, and managed runtime controls.

| Key      | Action                                        |
| -------- | --------------------------------------------- |
| `/`      | Edit config search                            |
| `Ctrl+U` | Clear search while editing                    |
| `S`      | Cycle sort field                              |
| `F`      | Cycle filter: all, enabled, failed, has-delay |
| `P`      | Cycle protocol filter                         |
| `T`      | Show or hide soft-deleted configs             |
| `Enter`  | Start the focused config                      |
| `e`, `x` | Enable or disable the focused config          |
| `d` …    | Soft-delete chord (see below)                 |
| `D` …    | Purge chord (see below)                       |
| `r` …    | Restore chord (see below)                     |
| `t` …    | Test chord (see Testing Strip)                |
| `K`      | Stop/disconnect the managed runtime           |
| `R`      | Restart the managed runtime                   |
| `y`      | Show a QR code for the focused config URI     |
| `c`      | Copy the focused config URI                   |

### Chord keys

On the Configs tab, `t`, `d`, `D`, and `r` are chord leaders: press the leader,
then a second key to pick the scope. The key bar shows the available second keys
while a chord is armed; `Esc` (or any unbound key) cancels it. Every destructive
action — single-row (`d d`, `D D`, `r r`) and multi-config alike — asks for an
inline `y/n` confirmation in the key bar; there are no confirmation modals.
Multi-config chords run as a single bulk database operation.

| Chord | Action                                              |
| ----- | --------------------------------------------------- |
| `d d` | Soft-delete the focused config (confirm)            |
| `d f` | Soft-delete all failed configs                      |
| `d v` | Soft-delete all visible (filtered) configs          |
| `d x` | Soft-delete all disabled configs                    |
| `D D` | Purge the focused config (confirm)                  |
| `D f` | Purge all failed configs                            |
| `D v` | Purge visible configs that are already soft-deleted |
| `D a` | Empty trash — purge every soft-deleted config       |
| `r r` | Restore the focused soft-deleted config             |
| `r v` | Restore visible configs that are soft-deleted       |
| `r a` | Restore every soft-deleted config                   |

Search matches the displayed config fields. Sorting can cycle through latency,
ID, name, protocol, subscription, last-tested time, and imported time. Deleted
configs are hidden by default; press `T` to include them.

The config detail panel shows the subscription a config belongs to (`#id name`)
or `none` for configs added directly. The Configs table title shows the active
subscription filter (`· sub:<name>` or `· sub:orphans`) when one is set from the
Subscriptions tab.

Soft delete hides a config from normal views and workflows. Purge permanently
deletes it. Both destructive actions require confirmation.

The Runtime panel shows the current managed runtime state, active config,
current task, proxy endpoint, available proxy engines (xray / sing-box), daemon
status and rotation schedule, config counts, and failure message when present.
The API subscription URL is shown only when the HTTP API is enabled; when the
API binds to `0.0.0.0`/`::` the panel shows the host's LAN IP instead of the
wildcard address. Runtime actions use the same runtime service as
`xrat connect`, `xrat disconnect`, and `xrat status`. The same runtime
prerequisites apply: the configured Xray/V2Ray binary must be available, runtime
paths must be writable, and daemon/runtime configuration must be valid.

Focus a config on the Configs tab and press `Enter` to start or switch the
runtime. Runtime operations run in the background and reload TUI data after
completion.

## Subscriptions Tab

The Subscriptions tab replaces the Configs table with the subscription list; the
right Detail panel then shows the focused subscription's metadata. The local
HTTP API base64 subscription URL is shown in the Runtime panel.

The table starts with two synthetic rows that act as filters for the Configs
tab:

- `All configs` — clear the subscription filter; the Configs tab shows every
  config.
- `Orphans` — show only configs that do not belong to any subscription (for
  example, configs added with `xrat add`).

Below them is one row per subscription. Focusing any row applies its filter to
the Configs tab live, with no confirmation step; switch back to the Configs tab
to browse the filtered set.

| Key | Action                                                  |
| --- | ------------------------------------------------------- |
| `r` | Refresh the focused subscription                        |
| `R` | Refresh all subscriptions with stored values            |
| `n` | Rename the focused subscription                         |
| `d` | Delete the focused subscription and its configs         |
| `y` | Show a QR code for the focused subscription URL         |
| `c` | Copy the focused subscription URL                       |
| `u` | Show a QR code for the HTTP API `/b64` subscription URL |
| `U` | Copy the HTTP API `/b64` subscription URL               |

Subscription actions apply to the focused subscription row; they are no-ops on
the `All configs` and `Orphans` rows.

The TUI does not import new subscriptions. Add subscriptions from the CLI with
[`xrat import <input>`](import.md) for subscriptions, files, and link lists, or
[`xrat add <link>`](import.md#add) for a single config link, then refresh them
here. This keeps every Subscriptions row a real, refreshable subscription
record.

Subscription refresh runs as a background task. While it runs, the Runtime card
shows live activity and the bottom bar shows completion summaries that
auto-hide. When refresh finishes, the TUI reloads database-backed data so both
tabs reflect the new state, including any configs removed by subscription
reconciliation.

## Testing Strip

A full-width Testing strip sits below the filter bar under both tabs. Its left
side summarizes the test scope and count, mode, and concurrency. Its right side
shows a live progress gauge while a batch is running, then summarizes nonzero
completed result counts as done, ok, and failed.

Test batches run TCP and real-delay tests with concurrency `4` and skip
download, upload, and ICMP stages. Tests use the `t` chord leader: `t t`
(focused), `t a` (all enabled), `t v` (visible), `t r` (failed), `t s` (stale),
and `t c` cancels a running batch.

While a batch is running, the gauge updates without blocking navigation.
Cancelling requests cooperative cancellation; the active operation reports
cancelled once the shared test executor observes the cancellation request.

## Runtime, Logs, and Help

The merged runtime panel summarizes runtime, database, subscription, API, and
config-count state alongside the active config. Both the runtime and logs cards
stay visible under both tabs.

The Logs card is tabbed. Focus it with `2`, `Tab`, or `Shift+Tab`, then switch
tabs:

| Key       | Action                                       |
| --------- | -------------------------------------------- |
| `[` / `]` | Cycle to the previous / next log tab         |
| `C l`     | Clear the active log view (view-only)        |
| `C s`     | Clear the stats view / counters (view-only)  |
| `C p`     | Clear all persisted events from the database |

| Tab          | Shows                                                       |
| ------------ | ----------------------------------------------------------- |
| xrat events  | Structured app/runtime events (same data as `xrat logs`)    |
| proxy engine | Parsed xray / sing-box engine logs for the latest session   |
| stats        | Live traffic: total ↓/↑, current rate, delay, and sparkline |
| api          | HTTP API requests recorded by the server (`source = api`)   |

The proxy engine and stats tab titles show the active engine and version.

The engine tab parses recognized xray **and** sing-box log lines into time,
level, source/component, and message columns; the active engine and version are
shown once in the card title instead of repeating per row. Generated sing-box
configs enable `log.timestamp` so its lines carry a timestamp. xray access logs
get an inferred `Info` level and their `[inbound >> outbound]` routing path in
the source column. stderr is styled as a warning, and unrecognized lines are
kept as raw messages with severity inferred from keywords. Access logs from
xrat's own stats polling (`[api >> api]`) are hidden as instrumentation noise.

Severity colors are shared across the events, api, and engine tabs:
critical/fatal/panic/error are red, warn/warning are yellow, and
info/debug/trace are neutral/accent.

The stats tab samples the active engine once per second — the xray gRPC
`StatsService` or the sing-box Clash API `/connections` endpoint — and resets
its history on each new runtime session. It is enabled by `[runtime.stats]`
(see [config reference](../05-reference/config-file.md)).

Clears come in two kinds. `C l` and `C s` are **view-only**: they hide the
current log/stats buffer in the TUI without deleting anything, and a periodic
reload does not resurrect the cleared rows. `C p` is a **database** clear — it
removes the persisted `events` rows, the same data cleared by
[`xrat logs clear`](logs.md#clearing-persisted-events). Engine log files are not
touched by any of them.

Press `?` from either tab to open the help modal. Press `Esc` to close it.

## QR and Clipboard Behavior

QR modals are available for focused config URIs, subscription URLs, and the HTTP
API subscription URL. Press `Esc` or `q` to close a QR modal.

Clipboard actions use the host clipboard. They can fail in SSH, tmux, Wayland,
X11, or headless sessions depending on environment support. When clipboard
access fails, the TUI reports the error in the status area.

QR generation can fail if a URI is too long for the QR renderer. When that
happens, the QR modal reports the failure instead of crashing.

## Related Commands

| Workflow              | CLI equivalent                                     |
| --------------------- | -------------------------------------------------- |
| Manage config state   | [`config management`](config-management.md)        |
| Start or stop runtime | [`runtime`](runtime.md)                            |
| Run tests             | [`test`](test.md)                                  |
| Inspect subscriptions | [`list subscriptions`](list.md#list-subscriptions) |
| Import subscriptions  | [`import`](import.md)                              |
| Refresh subscriptions | [`update`](update.md)                              |
| Serve API URL         | [`serve`](serve.md)                                |

## Troubleshooting

If the TUI cannot start, check that the terminal supports alternate-screen raw
mode and run with a higher log level:

```bash
xrat -vv tui
```

If runtime actions fail, verify the equivalent CLI flow first:

```bash
xrat daemon start
xrat connect <id>
xrat status
```

If subscription/API QR or copy actions report that a URL is unavailable, ensure
the subscription has a stored value and that the HTTP API subscription URL can
be built from the current app configuration.
