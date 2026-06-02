# tui

Start the interactive terminal UI.

```bash
xrat tui
```

## Flags

No command-specific flags.

The TUI is the interactive view over xrat's shared database, runtime, source,
and testing services. It is useful when you want to browse configs, inspect
current state, and run day-to-day operations without switching between many
commands.

## Current Capabilities

- Browse configs with latest test summaries
- Search, sort, and toggle deleted-row visibility
- Select, enable, disable, soft-delete, purge, and restore configs
- Inspect subscription sources
- Inspect runtime/session status
- Inspect latest test-run summaries and recent results
- Start scoped background test batches from the Tests view
- Cancel in-flight TUI test batches
- Refresh focused or all subscription sources
- Import a new source from the Sources view
- Start, stop, restart, and switch runtime configs from the Runtime view
- Show QR for a focused config or source URL
- Copy focused config/source URI, or selected config URIs, to the clipboard
- View diagnostics and help inside the terminal UI

Advanced protocol/source filters, HTTP API subscription URL QR/copy, selected
profile QR payloads, and broader environment verification are still Phase 6
polish areas. Use the CLI commands directly when you need a workflow that is not
yet wired in the TUI.

## Related Commands

| Workflow              | CLI equivalent                                     |
| --------------------- | -------------------------------------------------- |
| Manage config state   | [`config management`](config-management.md)        |
| Start or stop runtime | [`runtime`](runtime.md)                            |
| Run tests             | [`test`](test.md)                                  |
| Inspect sources       | [`list subscriptions`](list.md#list-subscriptions) |

## Troubleshooting

If the TUI cannot start, check that the terminal supports alternate-screen raw
mode and run with a higher log level:

```bash
xrat -vv tui
```

If runtime actions are not available in the TUI, use `xrat connect`,
`xrat disconnect`, or `xrat status` from the CLI.

Clipboard access can fail in SSH, tmux, Wayland, or headless sessions. When that
happens, the TUI reports the clipboard error in its status area.
