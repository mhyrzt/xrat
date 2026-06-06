# Daemon and IPC

xrat includes a long-lived daemon supervisor process that manages proxy
sessions, monitors health, and handles auto-rotation. CLI commands communicate
with the daemon via Unix domain socket IPC.

## Daemon Architecture

The daemon runs as a background process with three main responsibilities:

1. **IPC Server** — Listens for commands from CLI clients
2. **Health Monitor** — Periodically checks proxy liveness
3. **Rotation Scheduler** — Manages automatic proxy switching

### Supervisor Event Loop

The daemon runs a `tokio::select!` loop:

```rust
loop {
    tokio::select! {
        _ = health_tick.tick() => {
            check_proxy_health().await;
        }
        Some(event) = ipc_rx.recv() => {
            handle_ipc_event(event).await;
        }
        _ = rotation_timer.tick() => {
            trigger_rotation().await;
        }
    }
}
```

## IPC Protocol

The daemon uses JSON over Unix domain socket with protocol version 1.

### Socket Location

```
<runtime_dir>/daemon.sock
```

Default: `~/.config/xrat/runtime/daemon.sock`

### Connection Flow

1. CLI command connects to the Unix socket
2. Sends a JSON request
3. Receives a JSON response
4. Closes the connection

### Request Format

```json
{
  "protocol_version": 1,
  "request": {
    "type": "RuntimeConnect",
    "payload": {
      "config_id": 42
    }
  }
}
```

### Response Format

```json
{
  "protocol_version": 1,
  "ok": true,
  "code": 200,
  "message": "success",
  "payload": { ... }
}
```

### Request Types

| Type                | Description                   | Payload                     |
| ------------------- | ----------------------------- | --------------------------- |
| `DaemonPing`        | Check daemon reachability     | None                        |
| `DaemonShutdown`    | Request graceful shutdown     | None                        |
| `RuntimeStatus`     | Get proxy runtime status      | None                        |
| `RuntimeConnect`    | Start a proxy session         | `{ config_id: i64 }`        |
| `RuntimeReplace`    | Atomic disconnect + connect   | `{ trigger, candidate_id }` |
| `RuntimeDisconnect` | Stop the active proxy session | None                        |
| `ProxyStart`        | Enable auto-rotation          | None                        |
| `ProxyStatus`       | Get rotation status           | None                        |
| `ProxyStop`         | Disable auto-rotation         | None                        |

Manual `xrat rotate now` calls `RuntimeReplace` with `trigger = manual` and an
optional `candidate_id`. There is no separate `ProxyRotate` request type.

### Response Codes

| Code  | Description                        |
| ----- | ---------------------------------- |
| `200` | Success                            |
| `400` | Bad request (invalid payload)      |
| `404` | Not found (no active session)      |
| `409` | Conflict (session already running) |
| `500` | Internal error                     |

## Daemon Lifecycle

### Starting the Daemon

```bash
xrat daemon start
```

1. Check if daemon is already running (try connecting to socket)
2. Fork a background process
3. Create the Unix socket
4. Run the supervisor event loop
5. Reattach to any stale sessions from previous daemon runs

### Stopping the Daemon

```bash
xrat daemon stop
```

1. Connect to the daemon socket
2. Send `DaemonShutdown` request
3. Daemon gracefully terminates:
   - Stops the active proxy session (if running)
   - Closes the IPC socket
   - Exits cleanly

### Checking Daemon Status

```bash
xrat daemon status
```

Attempts to connect to the socket and sends a `DaemonPing` request. Reports
whether the daemon is reachable and the protocol version.

## Health Monitoring

The daemon periodically checks the health of the active proxy session.

### Health Tick Interval

Every 15 seconds, the daemon:

1. Loads the active session from the database
2. Checks if the PID is still running
3. Tests TCP reachability of the SOCKS port
4. If health check fails:
   - Logs the failure
   - Triggers auto-rotation (if `health_trigger_enabled`)

### Health Check Implementation

```rust
async fn check_proxy_health() -> HealthStatus {
    let session = db.get_latest_running_session().await?;

    if !process_is_running(session.process_id) {
        return HealthStatus::ProcessDead;
    }

    if !tcp_connect(session.socks_host, session.socks_port).await {
        return HealthStatus::PortUnreachable;
    }

    HealthStatus::Healthy
}
```

### Failure Handling

When a health check fails:

1. **Log the failure** — Record in daemon logs
2. **Update session** — Mark as `failed` with reason
3. **Trigger rotation** — If `health_trigger_enabled`, start rotation
4. **Respect cooldown** — Don't rotate if cooldown is active

## Session Reconciliation

When the daemon starts, it reconciles stale sessions from previous runs.

### Stale Session Detection

Query for sessions with:

- `status = 'running'`
- `stopped_at IS NULL`

### Reconciliation Logic

For each stale session:

1. **Check PID liveness** — Is the process still running?
2. **Verify process identity** — Does `/proc/<pid>/cmdline` match the expected
   binary?
3. **Decision**:
   - PID alive + cmdline matches → **reattach** (keep as `running`)
   - PID alive + cmdline mismatch → **mark failed** (different process reused
     PID)
   - PID dead → **mark failed**

### Reattach Validation

```rust
fn validate_reattach(pid: i64, expected_binary: &Path) -> bool {
    let cmdline_path = format!("/proc/{}/cmdline", pid);
    let cmdline = std::fs::read_to_string(cmdline_path).ok()?;
    cmdline.contains(expected_binary.to_str()?)
}
```

This prevents reattaching to a different process that happens to have the same
PID.

## IPC Client

CLI commands use an IPC client to communicate with the daemon.

### Connection Retry

The client attempts to connect with retry:

```rust
async fn connect_with_retry(socket_path: &Path, max_attempts: u32) -> Result<UnixStream> {
    for attempt in 1..=max_attempts {
        match UnixStream::connect(socket_path).await {
            Ok(stream) => return Ok(stream),
            Err(_) if attempt < max_attempts => {
                sleep(Duration::from_millis(100)).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### Error Handling

If the daemon is not running or unreachable:

```
Error: daemon socket not reachable at /home/user/.config/xrat/runtime/daemon.sock
Hint: start the daemon with 'xrat daemon start'
```

## Daemon Integration

### CLI Commands via IPC

When the daemon is running, these commands route through IPC:

| Command             | IPC Request         |
| ------------------- | ------------------- |
| `xrat connect <id>` | `RuntimeConnect`    |
| `xrat disconnect`   | `RuntimeDisconnect` |
| `xrat status`       | `RuntimeStatus`     |
| `xrat rotate start`  | `ProxyStart`        |
| `xrat rotate status` | `ProxyStatus`       |
| `xrat rotate now` | `RuntimeReplace`    |
| `xrat rotate stop`   | `ProxyStop`         |

### Daemon Required

Runtime and proxy commands require the daemon IPC path. If the daemon is not
running, `xrat connect`, `xrat status`, `xrat disconnect`, and `xrat proxy ...`
return a hint to start it:

```bash
xrat daemon start
```

## Supervisor State

The daemon maintains internal state:

```rust
struct SupervisorState {
    rotation_enabled: bool,
    last_rotation_at: Option<DateTime<Utc>>,
    last_trigger: Option<RotationTrigger>,
    cooldown_until: Option<DateTime<Utc>>,
    next_timer_at: Option<DateTime<Utc>>,
    instance_id: String,
}
```

### Instance ID

Each daemon instance has a unique ID (UUID v4) used to track session ownership:

```rust
let instance_id = Uuid::new_v4().to_string();
```

Sessions created by the daemon include:

```json
{
  "owner_kind": "daemon",
  "owner_instance_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

## Logging

The daemon logs to stderr and optionally to a file:

```bash
xrat daemon start 2> daemon.log
```

Or with systemd:

```ini
[Service]
StandardOutput=journal
StandardError=journal
```

### Log Levels

Control verbosity with `RUST_LOG`:

```bash
RUST_LOG=info xrat daemon start
RUST_LOG=debug xrat daemon start
```

## Security Considerations

### Socket Permissions

The Unix socket is created with default permissions (usually `0755`). On
multi-user systems, consider restricting access:

```bash
chmod 700 ~/.config/xrat/runtime/daemon.sock
```

### API Key

The daemon does not enforce authentication for IPC. Security relies on Unix
socket permissions and filesystem access control.

For the HTTP API (if enabled), use the `key` field in config.toml for
authentication.

## Troubleshooting

### Daemon Not Starting

**Symptom**: `xrat daemon start` fails or exits immediately

**Check**:

- Is a daemon already running? `xrat daemon status`
- Check logs: `RUST_LOG=debug xrat daemon start`
- Verify socket directory exists and is writable

### IPC Connection Failed

**Symptom**: CLI commands fail with "daemon socket not reachable"

**Check**:

- Is the daemon running? `ps aux | grep xrat`
- Does the socket file exist? `ls -la ~/.config/xrat/runtime/daemon.sock`
- Check socket permissions

### Stale Sessions

**Symptom**: `xrat status` shows a session but no proxy is running

**Fix**:

```bash
xrat disconnect
xrat daemon stop
xrat daemon start
```

The daemon will reconcile stale sessions on startup.

## Related

- [`daemon` CLI](../02-cli/daemon.md) — command reference
- [Auto-Rotation](auto-rotation.md) — rotation scheduling
- [Runtime Management](runtime-management.md) — session lifecycle
