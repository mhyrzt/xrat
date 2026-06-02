# daemon

Run or control the daemon supervisor process.

```bash
xrat daemon <action>
```

## Actions

| Action      | Description                                           |
| ----------- | ----------------------------------------------------- |
| `start`     | Start the long-lived daemon process                   |
| `status`    | Show daemon IPC reachability and protocol information |
| `stop`      | Request daemon shutdown via local IPC                 |
| `install`   | Install xrat-daemon.service as a systemd user service |
| `uninstall` | Remove the installed systemd user service             |

The hidden internal `run-server` action is used by the daemon launcher and is
not a user-facing command.

---

## daemon start

Start the long-lived daemon supervisor process.

```bash
xrat daemon start
```

### Flags

No command-specific flags.

### Behavior

1. Forks a background daemon process
2. Creates a Unix domain socket at `<runtime_dir>/daemon.sock`
3. Runs the supervisor event loop with:
   - Health checks every 15 seconds
   - IPC event processing from CLI commands
   - Auto-rotation scheduling (if enabled)
4. Reattaches to any stale runtime sessions from previous daemon runs

### Daemon Features

- **IPC server**: Listens for commands from `xrat connect`, `xrat disconnect`,
  etc.
- **Health monitoring**: Periodically checks proxy liveness, triggers rotation
  on failure
- **Auto-rotation**: Scheduled proxy switching with cooldown and candidate
  testing
- **Session reconciliation**: Detects and recovers from stale sessions on
  restart

---

## daemon status

Show daemon IPC reachability and protocol information.

```bash
xrat daemon status
```

### Flags

No command-specific flags.

### Output

```
Daemon Status
─────────────────────────────────
Socket:     /home/user/.config/xrat/runtime/daemon.sock
Reachable:  yes
Protocol:   v1
```

If the daemon is not running or the socket is unreachable, prints an error.

---

## daemon stop

Request daemon shutdown via local IPC.

```bash
xrat daemon stop
```

### Flags

No command-specific flags.

### Behavior

1. Connects to the daemon socket
2. Sends a shutdown request
3. Daemon gracefully terminates:
   - Stops the active proxy session (if running)
   - Closes the IPC socket
   - Exits cleanly

---

## daemon install

Install xrat as a systemd user service. Linux only.

```bash
xrat daemon install [--start] [--with-api] [--dry-run]
```

### Flags

| Flag         | Description                                                           |
| ------------ | --------------------------------------------------------------------- |
| `--start`    | Start the daemon immediately after enabling the service               |
| `--with-api` | Also install `xrat-api.service` (standalone HTTP API)                 |
| `--dry-run`  | Print the generated unit and planned actions without writing anything |

### Behavior

1. Resolves the current binary path via `std::env::current_exe()`
2. Generates `xrat-daemon.service` from the template in `packaging/systemd/`
   with the resolved binary path and configured XRAT root
3. Writes the service file to `~/.config/systemd/user/` (respects
   `$XDG_CONFIG_HOME`)
4. Runs `systemctl --user daemon-reload`
5. Runs `systemctl --user enable xrat-daemon.service`
6. If `--start`: runs `systemctl --user start xrat-daemon.service`
7. If `--with-api`: generates and installs `xrat-api.service` as well

### Example

```bash
xrat daemon install --start
```

```
Written: /home/user/.config/systemd/user/xrat-daemon.service
Reloaded systemd user daemon.
Enabled: xrat-daemon.service
Started: xrat-daemon.service

Daemon installed successfully.
```

### Dry run

```bash
xrat daemon install --dry-run
```

Prints the generated service unit and the systemctl commands that would run,
without writing any files or calling systemctl.

---

## daemon uninstall

Remove the installed xrat-daemon.service systemd user service.

```bash
xrat daemon uninstall [--dry-run]
```

### Flags

| Flag        | Description                                     |
| ----------- | ----------------------------------------------- |
| `--dry-run` | Print planned actions without removing anything |

### Behavior

1. Stops `xrat-daemon.service` (non-fatal if not running)
2. Disables `xrat-daemon.service`
3. Removes `~/.config/systemd/user/xrat-daemon.service`
4. Repeats for `xrat-api.service` if present
5. Runs `systemctl --user daemon-reload`

User config, database, logs, and all application state are **preserved**.

---

## IPC Protocol

The daemon uses JSON over Unix domain socket with protocol version 1.

### Request Types

| Request             | Description                         |
| ------------------- | ----------------------------------- |
| `DaemonPing`        | Check daemon reachability           |
| `DaemonShutdown`    | Request graceful shutdown           |
| `RuntimeStatus`     | Get proxy runtime status            |
| `RuntimeConnect`    | Start a proxy session               |
| `RuntimeReplace`    | Atomic disconnect-old + connect-new |
| `RuntimeDisconnect` | Stop the active proxy session       |
| `ProxyStart`        | Enable auto-rotation                |
| `ProxyStatus`       | Get rotation status                 |
| `ProxyStop`         | Disable auto-rotation               |

Manual proxy rotation uses `RuntimeReplace` with a manual trigger and optional
candidate config ID. There is no separate `ProxyRotate` IPC request type.

### Response Envelope

```json
{
  "protocol_version": 1,
  "ok": true,
  "code": 200,
  "message": "success",
  "payload": { ... }
}
```

## Related

- [`proxy`](proxy.md) — control auto-rotation scheduling
- [`connect`](runtime.md#connect) — start a proxy via daemon IPC
- [`status`](runtime.md#status) — check proxy status via daemon IPC
- [`init`](init.md) — initialize config directory before first use
- [systemd](../04-deployment/systemd.md) — full systemd deployment guide
