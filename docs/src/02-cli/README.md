# CLI Reference

xrat is a command-first CLI tool. All operations are invoked as subcommands.

## Global Flags

These flags apply to every command:

| Flag                | Description                                                          |
| ------------------- | -------------------------------------------------------------------- |
| `-v`, `--verbose`   | Increase log verbosity. Repeat: `-v`=info, `-vv`=debug, `-vvv`=trace |
| `-q`, `--quiet`     | Suppress output except errors. Ignored if `RUST_LOG` is set          |
| `--database <path>` | SQLite database path override                                        |
| `--config <path>`   | Config file path override                                            |
| `--xray <path>`     | Xray binary path override                                            |
| `--v2ray <path>`    | V2Ray binary path override                                           |
| `--sing-box <path>` | sing-box binary path override                                        |

## Commands

| Command                                   | Description                                                     |
| ----------------------------------------- | --------------------------------------------------------------- |
| [`import`](import.md)                     | Import a subscription URL, file, or raw text into the database  |
| [`add`](config-management.md#add)         | Add a single config URI directly to the database                |
| [`stable refs`](refs.md)                  | Use short stable refs instead of numeric database IDs           |
| [`list`](list.md)                         | List stored configs or subscription sources                     |
| [`show`](config-management.md#show)       | Show details for a stored config                                |
| [`enable`](config-management.md#enable)   | Include a config in normal operations                           |
| [`disable`](config-management.md#disable) | Exclude a config from normal operations                         |
| [`delete`](config-management.md#delete)   | Soft-delete or permanently delete a config                      |
| [`restore`](config-management.md#restore) | Restore a soft-deleted config                                   |
| [`parse`](parse.md)                       | Parse and validate config links without persisting              |
| [`test`](test.md)                         | Test connectivity and latency for stored configs                |
| [`scan`](scan.md)                         | Scan candidate IPs for TCP reachability                         |
| [`connect`](runtime.md#connect)           | Start a managed proxy runtime for a stored config               |
| [`disconnect`](runtime.md#disconnect)     | Stop the active managed proxy runtime                           |
| [`status`](runtime.md#status)             | Show the managed proxy runtime status                           |
| [`daemon`](daemon.md)                     | Run or control the daemon supervisor process                    |
| [`proxy`](proxy.md)                       | Control auto-rotating proxy scheduling via the daemon           |
| [`geoip`](geoip.md)                       | Manage GeoLite2 MMDB assets and inspect GeoIP backend config    |
| [`serve`](serve.md)                       | Start the local HTTP API server                                 |
| [`tui`](tui.md)                           | Start the interactive terminal UI                               |
| [`upgrade`](upgrade.md)                   | Self-upgrade from the latest release or by building from source |
| `version`                                 | Print the xrat version                                          |

## Common State Terms

These words appear across the CLI, TUI, API, and database:

| Term       | Meaning                                                    |
| ---------- | ---------------------------------------------------------- |
| `enabled`  | Included in bulk tests and rotation candidate sets         |
| `disabled` | Stored but normally skipped by filtered workflows          |
| `active`   | Config attached to the current managed runtime session     |
| `deleted`  | Soft-deleted row hidden from normal lists unless requested |

Use [`connect`](runtime.md#connect) to make a config active by starting a
runtime session.

## Logging

xrat uses `tracing` for structured logging. Control verbosity with:

- `-v` / `--verbose`: info level
- `-vv`: debug level
- `-vvv`: trace level
- `-q` / `--quiet`: error level only
- `RUST_LOG` environment variable: overrides all flags

Logs are written to stderr.
