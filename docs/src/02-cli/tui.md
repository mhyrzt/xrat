# tui

Start the interactive terminal UI.

```bash
xrat tui
```

The TUI has no command-specific flags. It uses the same global flags as other
commands, including `--database`, `--config`, `--xray`, `--v2ray`, `--sing-box`,
`-v`, and `-q`.

The TUI is an interactive view over xrat's shared database, source, testing,
runtime, and diagnostics services. It does not keep a separate copy of business
logic: config changes, imports, tests, and runtime operations use the same app
services as the CLI commands.

## Views

| Key | View        | Purpose                                                                         |
| --- | ----------- | ------------------------------------------------------------------------------- |
| `1` | Configs     | Browse, filter, start, enable, disable, delete, and share configs               |
| `2` | Sources     | Inspect subscription sources, refresh/import sources, and share source/API URLs |
| `3` | Tests       | Start/cancel background test batches and inspect recent results                 |

The TUI opens in the Configs view. The bottom bar shows view shortcuts, help and
quit actions, active filters, task state, and the latest status message.

## Global Keys

| Key           | Action                                |
| ------------- | ------------------------------------- |
| `1`-`3`       | Switch primary view                   |
| `Tab`         | Cycle primary views                   |
| `j`, `k`      | Move focus down/up                    |
| arrow keys    | Move focus down/up                    |
| `?`           | Open help                             |
| `Esc`         | Close modal, leave search, or go back |
| `q`, `Ctrl+C` | Quit                                  |

## Configs View

The Configs view shows stored configs with latest test summaries and config
state. The status marker column uses `●` for active, `✕` for soft-deleted, `○`
for disabled, and `!` for failed configs. Long names are truncated in the table.
It supports focused actions, test batches, and managed runtime controls.

| Key      | Action                                              |
| -------- | --------------------------------------------------- |
| `/`      | Edit config search                                  |
| `Ctrl+U` | Clear search while editing                          |
| `S`      | Cycle sort field                                    |
| `F`      | Cycle filter: all, enabled, failed, has-delay       |
| `P`      | Cycle protocol filter                               |
| `T`      | Show or hide soft-deleted configs                   |
| `Enter`  | Start the focused config                            |
| `e`, `x` | Enable or disable the focused config                |
| `d`      | Soft-delete the focused config after confirmation   |
| `D`      | Purge the focused config after confirmation         |
| `r`      | Restore the focused soft-deleted config             |
| `t`      | Start a test batch for the current Configs scope    |
| `a`      | Test all enabled, non-deleted configs               |
| `v`      | Test visible configs matching current filters       |
| `K`      | Stop/disconnect the managed runtime                 |
| `R`      | Restart the managed runtime                         |
| `y`      | Show a QR code for the focused config URI           |
| `c`      | Copy the focused config URI                         |

Search matches the displayed config fields. Sorting can cycle through latency,
ID, name, protocol, source, last-tested time, and imported time. Deleted configs
are hidden by default; press `T` to include them.

Soft delete hides a config from normal views and workflows. Purge permanently
deletes it. Both destructive actions require confirmation.

The Runtime panel inside the Configs view shows the current managed runtime
state, active config, current task, proxy endpoint, config counts, and failure
message when present. Runtime actions use the same runtime service as
`xrat connect`, `xrat disconnect`, and `xrat status`. The same runtime
prerequisites apply: the configured Xray/V2Ray binary must be available, runtime
paths must be writable, and daemon/runtime configuration must be valid.

Focus a config in the Configs view and press `Enter` to start or switch the
runtime. Runtime operations run in the background and reload TUI data after
completion.

## Sources View

The Sources view lists subscription sources and source metadata. The detail
panel also shows the local HTTP API base64 subscription URL when it is
available.

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

The import modal accepts the same input forms as `xrat import`: subscription
URL, file path, raw config link, raw link list, base64 subscription text, SIP008
JSON, or Xray JSON. Press `Enter` to import and `Esc` to cancel.

Source refresh and import run as background tasks. When they finish, the TUI
reloads database-backed data so the Configs and Sources views reflect the new
state.

## Tests View

The Tests view shows the latest test run summary, current test settings,
progress, and recent results.

| Key | Action                        |
| --- | ----------------------------- |
| `s` | Start a background test batch |
| `c` | Cancel the running test batch |

The current implementation starts a batch for all enabled, non-deleted configs.
It runs TCP and real-delay tests with concurrency `4` and skips download,
upload, and ICMP stages. The scope, mode, and concurrency are displayed in the
view, but there are not yet keybindings to change them interactively.

While a batch is running, the progress bar updates without blocking navigation.
Cancelling requests cooperative cancellation; the active operation reports
cancelled once the shared test executor observes the cancellation request.

## Runtime, Failures, and Help

The Configs view shows the merged runtime panel, which summarizes runtime,
database, source, API, and config-count state alongside the active and selected
configs. The adjacent Failures and Event Log panel lists per-config failures and
recent TUI task messages.

Press `?` from any view to open the help modal. Press `Esc` to close it.

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
