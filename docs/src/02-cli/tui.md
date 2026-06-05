# tui

Start the interactive terminal UI.

```bash
xrat tui
```

The TUI has no command-specific flags. It uses the same global flags as other
commands, including `--database`, `--config`, `--xray`, `--v2ray`, `--sing-box`,
`-v`, and `-q`.

The TUI is an interactive view over xrat's shared database, source, testing, and
runtime services. It does not keep a separate copy of business logic: config
changes, imports, tests, and runtime operations use the same app services as the
CLI commands.

## Tabs

The TUI is a single dashboard. The top-left table has two tabs; switching the
tab also swaps the detail panel on the right. The Testing strip, the Logs panel,
and the Runtime panel stay visible under both tabs.

| Tab     | Purpose                                                                         |
| ------- | ------------------------------------------------------------------------------- |
| Configs | Browse, filter, start, test, enable, disable, delete, and share configs         |
| Sources | Inspect subscription sources, refresh/import sources, and share source/API URLs |

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
| `j`, `k`      | Move row / scroll the focused card down/up        |
| arrow keys    | Move row / scroll the focused card down/up        |
| `?`           | Open help                                         |
| `Esc`         | Close modal, leave search, or go back             |
| `q`, `Ctrl+C` | Quit                                              |

## Cards and Scrolling

The dashboard has four cards: the table (Configs/Sources), the detail panel,
Logs, and Runtime. `Tab` / `Shift+Tab` move focus between them; the focused card
is drawn with an accent border. `j`/`k` (or the arrow keys) move the row
selection when the table is focused, and scroll the focused card otherwise.
Cards that overflow their height show a scrollbar.

## Configs Tab

The Configs tab shows stored configs with latest test summaries and config
state. The status marker column uses `●` for active, `✕` for soft-deleted, `○`
for disabled, and `!` for failed configs. Long names are truncated in the table.
It supports focused actions, test batches, and managed runtime controls.

| Key      | Action                                            |
| -------- | ------------------------------------------------- |
| `/`      | Edit config search                                |
| `Ctrl+U` | Clear search while editing                        |
| `S`      | Cycle sort field                                  |
| `F`      | Cycle filter: all, enabled, failed, has-delay     |
| `P`      | Cycle protocol filter                             |
| `T`      | Show or hide soft-deleted configs                 |
| `Enter`  | Start the focused config                          |
| `e`, `x` | Enable or disable the focused config              |
| `d`      | Soft-delete the focused config after confirmation |
| `D`      | Purge the focused config after confirmation       |
| `r`      | Restore the focused soft-deleted config           |
| `t`      | Start a test batch for the current Configs scope  |
| `a`      | Test all enabled, non-deleted configs             |
| `v`      | Test visible configs matching current filters     |
| `C`      | Cancel the running test batch                     |
| `K`      | Stop/disconnect the managed runtime               |
| `R`      | Restart the managed runtime                       |
| `y`      | Show a QR code for the focused config URI         |
| `c`      | Copy the focused config URI                       |

Search matches the displayed config fields. Sorting can cycle through latency,
ID, name, protocol, source, last-tested time, and imported time. Deleted configs
are hidden by default; press `T` to include them.

The config detail panel shows the subscription a config belongs to (`#id name`)
or `none` for configs added directly. The Configs table title shows the active
source filter (`· src:<name>` or `· src:orphans`) when one is set from the
Sources tab.

Soft delete hides a config from normal views and workflows. Purge permanently
deletes it. Both destructive actions require confirmation.

The Runtime panel shows the current managed runtime state, active config,
current task, proxy endpoint, available proxy engines (xray / sing-box), daemon
status and rotation schedule, config counts, and failure message when present.
The API subscription URL is shown only when the HTTP API is enabled; when the
API binds to `0.0.0.0`/`::` the panel shows the host's LAN IP instead of the
wildcard address. Runtime actions use the same runtime service as `xrat connect`,
`xrat disconnect`, and `xrat status`. The same runtime prerequisites apply: the
configured Xray/V2Ray binary must be available, runtime paths must be writable,
and daemon/runtime configuration must be valid.

Focus a config on the Configs tab and press `Enter` to start or switch the
runtime. Runtime operations run in the background and reload TUI data after
completion.

## Sources Tab

The Sources tab replaces the Configs table with the subscription list; the right
Detail panel then shows the focused subscription's metadata. The local HTTP API
base64 subscription URL is shown in the Runtime panel.

The table starts with two synthetic rows that act as filters for the Configs
tab:

- `All configs` — clear the source filter; the Configs tab shows every config.
- `Orphans` — show only configs that do not belong to any subscription (for
  example, configs added with `xrat add`).

Below them is one row per subscription. Focusing any row applies its filter to
the Configs tab live, with no confirmation step; switch back to the Configs tab
to browse the filtered set.

| Key | Action                                                  |
| --- | ------------------------------------------------------- |
| `r` | Refresh the focused source                              |
| `R` | Refresh all sources with stored values                  |
| `i` | Open the import modal                                   |
| `n` | Rename the focused source                               |
| `d` | Delete the focused source and its configs               |
| `y` | Show a QR code for the focused source URL               |
| `c` | Copy the focused source URL                             |
| `u` | Show a QR code for the HTTP API `/b64` subscription URL |
| `U` | Copy the HTTP API `/b64` subscription URL               |

Source actions apply to the focused subscription row; they are no-ops on the
`All configs` and `Orphans` rows.

The import modal accepts the same input forms as `xrat import`: subscription
URL, file path, raw config link, raw link list, base64 subscription text, SIP008
JSON, or Xray JSON. Press `Enter` to import and `Esc` to cancel.

Source refresh and import run as background tasks. When they finish, the TUI
reloads database-backed data so both tabs reflect the new state.

## Testing Strip

A full-width Testing strip sits below the filter bar under both tabs. Its left
side summarizes the test scope and count, mode, and concurrency. Its right side
shows a live progress gauge while a batch is running, then summarizes nonzero
completed result counts as done, ok, and failed.

Test batches run TCP and real-delay tests with concurrency `4` and skip
download, upload, and ICMP stages. Start a batch with `t` (current scope), `a`
(all enabled), or `v` (visible), and cancel a running batch with `C`.

While a batch is running, the gauge updates without blocking navigation.
Cancelling requests cooperative cancellation; the active operation reports
cancelled once the shared test executor observes the cancellation request.

## Runtime, Logs, and Help

The merged runtime panel summarizes runtime, database, source, API, and
config-count state alongside the active config. The adjacent Logs panel lists
per-config failures and recent TUI task messages. Both panels stay visible under
both tabs.

Press `?` from either tab to open the help modal. Press `Esc` to close it.

## QR and Clipboard Behavior

QR modals are available for focused config URIs, source URLs, and the HTTP API
subscription URL. Press `Esc` or `q` to close a QR modal.

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
| Inspect sources       | [`list subscriptions`](list.md#list-subscriptions) |
| Import sources        | [`import`](import.md)                              |
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

If source/API QR or copy actions report that a URL is unavailable, ensure the
source has a stored value and that the HTTP API subscription URL can be built
from the current app configuration.
