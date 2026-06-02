# init

Initialize the xrat config directory, config file, and database.

```bash
xrat init [--dry-run]
```

## Flags

| Flag        | Description                                    |
| ----------- | ---------------------------------------------- |
| `--dry-run` | Print planned actions without creating anything |

## Behavior

1. Creates the app root directory (`$HOME/.config/xrat/` or `$XRAT_PATH`)
2. Writes a default `config.toml` with sensible defaults if not already present
3. Creates the SQLite database and runs all pending migrations
4. Creates subdirectories: `runtime/`, `logs/`, `mmdb/`
5. Prints a summary of what was created and what was already present

**Idempotent**: safe to run multiple times. Existing files are never
overwritten. If `config.toml` exists and has been customized, it is left
untouched.

## Example: first-time setup

```bash
xrat init
```

```
xrat initialized successfully.

Created:
  /home/user/.config/xrat/
  /home/user/.config/xrat/config.toml (written default template)
  /home/user/.config/xrat/runtime/
  /home/user/.config/xrat/logs/
  /home/user/.config/xrat/mmdb/

Already present:
  /home/user/.config/xrat/db.sqlite (database ready)

Next steps:
  xrat import <subscription-url>
  xrat list configs
```

## Example: dry run

```bash
xrat init --dry-run
```

```
--- dry run: no files written ---

Would create (if absent):
  /home/user/.config/xrat/
  /home/user/.config/xrat/config.toml
  /home/user/.config/xrat/db.sqlite
  /home/user/.config/xrat/runtime/
  /home/user/.config/xrat/logs/
  /home/user/.config/xrat/mmdb/
```

## State paths

| Path                             | Purpose                      | Override            |
| -------------------------------- | ---------------------------- | ------------------- |
| `$HOME/.config/xrat/`            | App root                     | `XRAT_PATH` env var |
| `$HOME/.config/xrat/config.toml` | Configuration                | `--config` flag     |
| `$HOME/.config/xrat/db.sqlite`   | SQLite database              | `--database` flag   |
| `$HOME/.config/xrat/runtime/`    | Daemon socket, session state | —                   |
| `$HOME/.config/xrat/logs/`       | Runtime logs                 | `[runtime.log].dir` |
| `$HOME/.config/xrat/mmdb/`       | GeoIP data                   | `[mmdb].dir`        |

## Default config.toml

The generated config enables SOCKS5 on port 1080, sets the Xray engine, and
provides commented-out HTTP inbound and log settings:

```toml
[runtime]
engine = "xray"

[runtime.socks]
enabled = true
host = "127.0.0.1"
port = 1080

[runtime.http]
enabled = false
host = "127.0.0.1"
port = 8080

[runtime.log]
enabled = true
level = "warning"

[testing]
concurrency = 4

[geo]
auto_update = false
```

See [Config File](../05-reference/config-file.md) for full reference.

## Related

- [Quickstart](../01-getting-started/quickstart.md)
- [Configuration](../01-getting-started/configuration.md)
- [daemon install](daemon.md#daemon-install) — install as a systemd user service
