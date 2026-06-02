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
| `1` | Configs     | Browse, filter, select, enable, disable, delete, and share configs              |
| `2` | Sources     | Inspect subscription sources, refresh/import sources, and share source/API URLs |
| `3` | Tests       | Start/cancel background test batches and inspect recent results                 |
| `4` | Runtime     | Inspect and control the managed runtime session                                 |
| `5` | Diagnostics | Inspect paths, runtime/source summaries, and recent operation messages          |

The TUI opens in the Configs view. The status bar shows the active view, config
counts, active filters, task state, and the latest status message.

## Global Keys

| Key           | Action                                |
| ------------- | ------------------------------------- |
| `1`-`5`       | Switch primary view                   |
| `j`, `k`      | Move focus down/up                    |
| arrow keys    | Move focus down/up                    |
| `?`           | Open help                             |
| `Esc`         | Close modal, leave search, or go back |
| `q`, `Ctrl+C` | Quit                                  |

## Configs View

The Configs view shows stored configs with latest test summaries and config
state. It supports focused actions and selected-config bulk actions.

| Key      | Action                                              |
| -------- | --------------------------------------------------- |
| `/`      | Edit config search                                  |
| `Ctrl+U` | Clear search while editing                          |
| `s`      | Cycle sort field                                    |
| `F`      | Cycle filter: all, enabled, failed, has-delay       |
| `P`      | Cycle protocol filter                               |
| `f`      | Show or hide soft-deleted configs                   |
| `Space`  | Mark the focused config as selected                 |
| `e`, `x` | Enable or disable the focused config                |
| `E`, `X` | Enable or disable all selected configs              |
| `d`      | Soft-delete the focused config after confirmation   |
| `D`      | Purge the focused config after confirmation         |
| `r`      | Restore the focused soft-deleted config             |
| `y`      | Show a QR code for the focused config URI           |
| `c`      | Copy the focused config URI                         |
| `C`      | Copy selected config URIs as newline-separated text |

Search matches the displayed config fields. Sorting can cycle through latency,
ID, name, protocol, source, last-tested time, and imported time. Deleted configs
are hidden by default; press `f` to include them.

Soft delete hides a config from normal views and workflows. Purge permanently
deletes it. Both destructive actions require confirmation.

## Sources View

The Sources view lists subscription sources and source metadata. The detail
panel also shows the local HTTP API base64 subscription URL when it is
available.

| Key | Action                                                  |
| --- | ------------------------------------------------------- |
| `r` | Refresh the focused source                              |
| `R` | Refresh all sources with stored values                  |
| `i` | Open the import modal                                   |
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

## Runtime View

The Runtime view shows the current managed runtime state, session details, PID
status, active config, selected config, inbound addresses, timestamps, and
failure or transition messages.

| Key | Action                                                                         |
| --- | ------------------------------------------------------------------------------ |
| `s` | Start/connect using the selected config, or active config if present           |
| `x` | Stop/disconnect the runtime                                                    |
| `r` | Restart using the active config, or selected config if no active config exists |
| `w` | Switch runtime to the selected config                                          |

Runtime actions use the same runtime service as `xrat connect`,
`xrat disconnect`, and `xrat status`. The same runtime prerequisites apply: the
configured Xray/V2Ray binary must be available, runtime paths must be writable,
and daemon/runtime configuration must be valid.

Select a preferred config in the Configs view before starting or switching the
runtime. Runtime operations run in the background and reload TUI data after
completion.

## Diagnostics and Help

Press `5` to open Diagnostics. It summarizes important runtime, database,
source, API, and operation state, including recent TUI task messages.

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
