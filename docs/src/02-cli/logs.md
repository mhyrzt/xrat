# logs

Show a unified view of application events and proxy engine logs.

```bash
xrat logs [flags]
```

`xrat logs` merges two sources:

- **App events** — structured rows recorded in the database (`events` table):
  daemon start/stop, runtime connect/disconnect, proxy rotation, health
  failover, and test runs.
- **Engine logs** — the stdout/stderr of the `xray-core` / `sing-box` process
  for the active or most recent runtime session, plus the daemon's own
  `daemon.log` file.

By default it prints the last N entries and exits. Use `--follow` to stream new
entries live (press `Ctrl-C` to stop).

## Flags

| Flag             | Description                                                                        |
| ---------------- | ---------------------------------------------------------------------------------- |
| `-f`, `--follow` | Stream new entries as they arrive instead of exiting                               |
| `-n`, `--lines`  | Number of recent entries to show before following (default: 200)                   |
| `--source`       | Which feeds to include: `all`, `app`, `daemon`, `xray`, `singbox` (default: `all`) |
| `--level`        | Minimum event level: `info`, `warn`, or `error` (applies to app events)            |
| `--format`       | Event stream format: `table`, `tsv`, or `json` (default: `table`)                  |

Notes:

- `--format json` / `--format tsv` emit the structured **app events** only;
  engine/daemon text logs are unstructured and are shown only in the default
  `table` view or while following.
- `--source xray` / `--source singbox` tail the engine log files for the active
  or last session; `--source daemon` tails `daemon.log`; `--source app` shows
  only structured events.

## Examples

```bash
# Last 200 events plus engine log tails
xrat logs

# Live stream everything
xrat logs -f

# Only the last 50 lines of xray-core output
xrat logs --source xray -n 50

# Only error-level app events, as JSON
xrat logs --source app --level error --format json
```

## Clearing persisted events

```bash
xrat logs clear [--yes]
```

`xrat logs clear` permanently deletes every row from the `events` table. It
prompts for confirmation first; pass `--yes` to skip the prompt (useful in
scripts). This only clears the structured **app events** in the database —
engine and daemon log files are left untouched, since they rotate with their
runtime sessions.

The TUI exposes the same database clear from the logs card via the `C p` clear
chord, kept distinct from any view-only buffer clears.

```bash
# Wipe all recorded app events without a prompt
xrat logs clear --yes
```

## Where logs live

| Source     | Location                                                  |
| ---------- | --------------------------------------------------------- |
| App events | `events` table in the database                            |
| Daemon     | `<runtime-dir>/daemon.log`                                |
| xray-core  | `<runtime-dir>/session-<id>.out.log` / `.err.log`         |
| sing-box   | `<runtime-dir>/session-<id>.singbox.out.log` / `.err.log` |

The daemon emits its own process output to `daemon.log` (errors and panics);
normal operational events are captured as structured rows instead.

## Related

- [`daemon`](daemon.md) — start/stop the supervisor that records most events
- [`proxy`](proxy.md) — auto-rotation, a frequent source of events
- [`runtime`](runtime.md) — connect, disconnect, and inspect active sessions
