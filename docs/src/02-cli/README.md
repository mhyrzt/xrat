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

| Command                               | Description                                                    |
| ------------------------------------- | -------------------------------------------------------------- |
| [`import`](import.md)                 | Import a subscription URL, file, or raw text into the database |
| [`add`](import.md#add)                | Add a single config URI directly to the database               |
| [`list`](list.md)                     | List stored configs or subscription sources                    |
| [`parse`](parse.md)                   | Parse and validate config links without persisting             |
| [`test`](test.md)                     | Test connectivity and latency for stored configs               |
| [`scan`](scan.md)                     | Scan candidate IPs for TCP reachability                        |
| [`connect`](runtime.md#connect)       | Start a managed proxy runtime for a stored config              |
| [`disconnect`](runtime.md#disconnect) | Stop the active managed proxy runtime                          |
| [`status`](runtime.md#status)         | Show the managed proxy runtime status                          |
| [`daemon`](daemon.md)                 | Run or control the daemon supervisor process                   |
| [`proxy`](proxy.md)                   | Control auto-rotating proxy scheduling via the daemon          |
| [`serve`](serve.md)                   | Start the local HTTP API server                                |

## Logging

xrat uses `tracing` for structured logging. Control verbosity with:

- `-v` / `--verbose`: info level
- `-vv`: debug level
- `-vvv`: trace level
- `-q` / `--quiet`: error level only
- `RUST_LOG` environment variable: overrides all flags

Logs are written to stderr.
