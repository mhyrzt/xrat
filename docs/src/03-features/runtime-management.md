# Runtime Management

xrat manages the lifecycle of local proxy processes (Xray or V2Ray), providing
automatic config generation, process spawning, health monitoring, and graceful
shutdown.

## Connect Flow

When you run `xrat connect <id>`:

1. **Load config** — Fetch the config from the database by ID
2. **Generate runtime config** — Create Xray JSON with local inbounds
3. **Spawn process** — Launch Xray/V2Ray as a child process
4. **Wait for readiness** — Poll the SOCKS port until it accepts connections
5. **Persist session** — Insert a `runtime_sessions` record with status
   `running`
6. **Return result** — Print connection details (ports, PID, config info)

### Runtime Config Generation

xrat generates a complete Xray config with:

- **Inbounds**: SOCKS5, HTTP, Shadowsocks (as configured in config.toml)
- **Outbound**: Single outbound to the proxy node
- **Logging**: Configurable log level and file paths
- **Stream settings**: TLS, WebSocket, gRPC, TCP header obfuscation

Example generated config:

```json
{
  "log": { "loglevel": "warning" },
  "inbounds": [
    {
      "tag": "socks-in",
      "port": 18200,
      "listen": "0.0.0.0",
      "protocol": "socks",
      "settings": { "udp": true }
    },
    {
      "tag": "http-in",
      "port": 18201,
      "listen": "0.0.0.0",
      "protocol": "http"
    }
  ],
  "outbounds": [
    {
      "tag": "proxy",
      "protocol": "vless",
      "settings": {
        "vnext": [
          {
            "address": "example.com",
            "port": 443,
            "users": [{ "id": "uuid-123", "encryption": "none" }]
          }
        ]
      },
      "stream_settings": {
        "network": "ws",
        "security": "tls",
        "tls_settings": { "server_name": "cdn.example.com" },
        "ws_settings": {
          "path": "/ray",
          "headers": { "Host": "cdn.example.com" }
        }
      }
    }
  ]
}
```

### Process Spawning

xrat spawns the proxy process with:

- **Config file**: Written to `<runtime_dir>/session-<id>.json`
- **Stdout**: Redirected to `<runtime_dir>/session-<id>.out.log`
- **Stderr**: Redirected to `<runtime_dir>/session-<id>.err.log`
- **Detached mode**: Process continues running after CLI exits (when using
  daemon)

### Readiness Check

After spawning, xrat polls the SOCKS port every 100ms until:

- Port accepts TCP connections → success
- Process exits → error
- Timeout (default 10s) → error, process is killed

## Session State

Each runtime session has a status:

| Status     | Description                                   |
| ---------- | --------------------------------------------- |
| `starting` | Process spawned, waiting for port readiness   |
| `running`  | Port is ready, proxy is active                |
| `stopping` | Graceful shutdown in progress                 |
| `stopped`  | Process terminated cleanly                    |
| `failed`   | Process exited unexpectedly or startup failed |

### State Transitions

```
starting → running → stopping → stopped
   ↓                      ↓
 failed                failed
```

### Session Record

Persisted to `runtime_sessions` table:

| Field                                  | Description                          |
| -------------------------------------- | ------------------------------------ |
| `id`                                   | Session ID (primary key)             |
| `config_id`                            | Foreign key to configs table         |
| `status`                               | Current status                       |
| `process_id`                           | OS process ID (PID)                  |
| `socks_host`, `socks_port`             | SOCKS inbound address                |
| `http_host`, `http_port`               | HTTP inbound address                 |
| `shadowsocks_host`, `shadowsocks_port` | Shadowsocks inbound address          |
| `failure_reason`                       | Error message (if failed)            |
| `owner_kind`                           | `cli` or `daemon`                    |
| `owner_instance_id`                    | Daemon instance ID (if daemon-owned) |
| `started_at`, `stopped_at`             | Timestamps                           |

## Disconnect Flow

When you run `xrat disconnect`:

1. **Load active session** — Find the latest `running` session
2. **Send SIGTERM** — Request graceful shutdown
3. **Wait for exit** — Poll process status every 100ms (up to 5s)
4. **Send SIGKILL** — Force kill if still running after timeout
5. **Update session** — Set status to `stopped` or `failed`
6. **Cleanup** — Remove temporary config files (if configured)

### Graceful Shutdown

xrat attempts graceful shutdown:

```rust
terminate_process_gracefully(pid, Duration::from_secs(5))
```

1. Check if process is running
2. Send SIGTERM
3. Poll every 100ms for up to 5 seconds
4. If still running, send SIGKILL
5. Return outcome: `Terminated`, `Killed`, or `NotRunning`

## Status Check

When you run `xrat status`:

1. **Load active session** — Find the latest session (any status)
2. **Check PID liveness** — Verify process is still running
3. **Check inbound health** — Test TCP reachability of SOCKS/HTTP/Shadowsocks
   ports
4. **Return snapshot** — Print status with config details and health

### Health Check

For each inbound port:

| Status        | Description                      |
| ------------- | -------------------------------- |
| `reachable`   | TCP connection succeeded         |
| `unreachable` | TCP connection failed            |
| `not_checked` | Inbound is disabled or port is 0 |

## Session Replacement

When `replace_active_session = true` in config.toml:

```bash
xrat connect a1b2
```

If a session is already running:

1. Disconnect the old session (graceful shutdown)
2. Connect the new session
3. Atomic operation from the user's perspective

This is useful for switching proxies without manual disconnect.

## Reattach on Daemon Restart

When the daemon starts, it reconciles stale sessions:

1. **Find stale sessions** — Query for `running` sessions with no `stopped_at`
2. **Check PID liveness** — For each stale session, check if PID is still
   running
3. **Verify process identity** — Compare the process executable and command
   line (queried via `sysinfo`, so it works across Linux/macOS/BSD) with the
   expected runtime engine and session config
4. **Reattach or mark failed**:
   - PID alive + cmdline matches → reattach (keep as `running`)
   - PID alive + cmdline mismatch → mark as `failed` (different process reused
     PID)
   - PID dead → mark as `failed`, then **auto-recover**

### Stale PID Recovery After Reboot

A dead PID is the common case after a reboot: the persisted session points at a
proxy process that no longer exists. Rather than leaving the runtime stopped and
forcing a manual reconnect, the daemon clears the stale attachment and
relaunches the persisted config automatically (when it is still enabled and not
deleted).

Recovery is recorded as an event visible in `xrat logs`:

- `daemon_restart_stale_pid_recovered` — the persisted config reconnected
  successfully.
- `daemon_restart_stale_pid_recovery_failed` — the relaunch attempt failed; the
  detail field carries the error.

A cmdline/exec mismatch is **not** auto-recovered, because a different live
process owns that PID and launching over it could be unsafe.

### Reattach Validation

xrat validates that the PID still belongs to the expected proxy process:

```rust
fn validate_reattach(pid: i64, expected_binary: &Path) -> bool {
    let cmdline = read_proc_cmdline(pid);
    cmdline.contains(expected_binary.to_str().unwrap())
}
```

This prevents reattaching to a different process that happens to have the same
PID.

## Inbound Configuration

Configure local inbounds in `config.toml`:

### SOCKS5

```toml
[runtime.socks]
enabled = true
host = "0.0.0.0"
port = 18200
udp = true
auth = { enabled = true, username = "xrat", password = { env = "XRAT_SOCKS_PASSWORD" } }
```

| Field     | Description                               |
| --------- | ----------------------------------------- |
| `enabled` | Enable SOCKS inbound                      |
| `host`    | Bind address                              |
| `port`    | Bind port                                 |
| `udp`     | Enable UDP support                        |
| `auth`    | Optional username/password authentication |

### HTTP

```toml
[runtime.http]
enabled = false
host = "0.0.0.0"
port = 18201
```

### Shadowsocks

```toml
[runtime.shadowsocks]
enabled = false
host = "0.0.0.0"
port = 18202
method = "aes-128-gcm"
password = { env = "XRAT_SHADOWSOCKS_PASSWORD" }
network = "tcp,udp"
```

## Sniffing

Enable traffic sniffing for better routing:

```toml
[runtime.sniffing]
enabled = true
dest_override = ["http", "tls", "quic"]
route_only = true
metadata_only = false
domains_excluded = []
ips_excluded = []
```

## Logging

Configure proxy process logging:

```toml
[runtime.log]
enabled = true
mask = "none"  # "quarter" | "half" | "full" | "none"
dir = "logs"
dns_log = false
level = "warning"  # "debug" | "info" | "warning" | "error"
keep = true
```

| Field     | Description                                        |
| --------- | -------------------------------------------------- |
| `enabled` | Enable logging to files                            |
| `mask`    | Mask IP addresses in logs                          |
| `dir`     | Log directory (relative to config dir or absolute) |
| `dns_log` | Enable DNS query logging                           |
| `level`   | Log level                                          |
| `keep`    | Keep log files after session stops                 |

## Engine Selection

Choose the proxy engine in `config.toml`:

```toml
[runtime]
engine = "xray"  # "xray" | "v2ray" | "sing-box"
```

| Engine     | Binary     | Protocols                                                                                   |
| ---------- | ---------- | ------------------------------------------------------------------------------------------- |
| `xray`     | `xray`     | All except Hysteria2                                                                        |
| `v2ray`    | `v2ray`    | VLESS, VMess, Shadowsocks, Trojan, HTTP, SOCKS5                                             |
| `sing-box` | `sing-box` | Managed Hysteria2 runtime sessions; other protocols currently require Xray/V2Ray generators |

Hysteria2 (`hy2`) configs are selected for sing-box automatically, even when
`engine = "xray"`, because Xray/V2Ray cannot generate a compatible Hysteria2
runtime config. Non-Hysteria2 configs with `engine = "sing-box"` fail with an
unsupported-combination error until their sing-box runtime generators are added.

## Related

- [`connect` CLI](../02-cli/runtime.md#connect) — command reference
- [Daemon and IPC](daemon-and-ipc.md) — daemon-managed sessions
- [Auto-Rotation](auto-rotation.md) — automatic proxy switching
- [Config Generation](../06-architecture/config-generation.md) — how configs are
  generated
