# daemon

Run or control the daemon supervisor process.

```bash
xrat daemon <action>
```

## Actions

| Action | Description |
|--------|-------------|
| `start` | Start the long-lived daemon process |
| `status` | Show daemon IPC reachability and protocol information |
| `stop` | Request daemon shutdown via local IPC |

---

## daemon start

Start the long-lived daemon supervisor process.

```bash
xrat daemon start
```

### Behavior

1. Forks a background daemon process
2. Creates a Unix domain socket at `<runtime_dir>/daemon.sock`
3. Runs the supervisor event loop with:
   - Health checks every 15 seconds
   - IPC event processing from CLI commands
   - Auto-rotation scheduling (if enabled)
4. Reattaches to any stale runtime sessions from previous daemon runs

### Daemon Features

- **IPC server**: Listens for commands from `xrat connect`, `xrat disconnect`, etc.
- **Health monitoring**: Periodically checks proxy liveness, triggers rotation on failure
- **Auto-rotation**: Scheduled proxy switching with cooldown and candidate testing
- **Session reconciliation**: Detects and recovers from stale sessions on restart

---

## daemon status

Show daemon IPC reachability and protocol information.

```bash
xrat daemon status
```

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

### Behavior

1. Connects to the daemon socket
2. Sends a shutdown request
3. Daemon gracefully terminates:
   - Stops the active proxy session (if running)
   - Closes the IPC socket
   - Exits cleanly

## IPC Protocol

The daemon uses JSON over Unix domain socket with protocol version 1.

### Request Types

| Request | Description |
|---------|-------------|
| `DaemonPing` | Check daemon reachability |
| `DaemonShutdown` | Request graceful shutdown |
| `RuntimeStatus` | Get proxy runtime status |
| `RuntimeConnect` | Start a proxy session |
| `RuntimeReplace` | Atomic disconnect-old + connect-new |
| `RuntimeDisconnect` | Stop the active proxy session |
| `ProxyStart` | Enable auto-rotation |
| `ProxyStatus` | Get rotation status |
| `ProxyStop` | Disable auto-rotation |

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
- [`connect`](runtime.md#connect) — start a proxy (via daemon IPC)
- [`status`](runtime.md#status) — check proxy status (via daemon IPC)
