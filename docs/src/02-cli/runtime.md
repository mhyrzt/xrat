# Runtime Commands

Manage the local proxy runtime through the daemon: connect, disconnect, and
check status.

---

## connect

Start a managed proxy runtime for a stored config.

```bash
xrat connect <ref> [flags]
```

### Arguments

| Argument | Description                                      |
| -------- | ------------------------------------------------ |
| `ref`    | Config ref prefix to start as the active session |

### Flags

| Flag     | Description              |
| -------- | ------------------------ |
| `--json` | Print the result as JSON |

### Examples

```bash
xrat connect a1b2
```

```bash
xrat connect a1b2c3d4 --json
```

### Behavior

1. Sends a runtime-connect request to the daemon over local IPC
2. The daemon loads the config from the database
3. Generates an Xray (or V2Ray) runtime config with local inbounds
4. Spawns the proxy process
5. Waits for the SOCKS port to become ready
6. Persists a `runtime_sessions` record with status `running`
7. Prints connection details

If the daemon is not running, start it first:

```bash
xrat daemon start
```

### Default Inbounds

| Protocol    | Host      | Port    | Notes                              |
| ----------- | --------- | ------- | ---------------------------------- |
| SOCKS5      | `0.0.0.0` | `18200` | UDP support enabled by default     |
| HTTP        | `0.0.0.0` | `18201` | Disabled by default in config.toml |
| Shadowsocks | `0.0.0.0` | `18202` | Disabled by default, aes-128-gcm   |

Configure inbounds in `config.toml` under `[runtime.socks]`, `[runtime.http]`,
and `[runtime.shadowsocks]`.

### Session Replacement

If `replace_active_session = true` in config.toml, connecting to a new config
automatically disconnects the previous session.

### Engine Boundary

The managed runtime uses Xray/V2Ray for protocols they can generate. Hysteria2
(`hy2`) configs are sing-box-only, so `xrat connect` automatically launches the
configured `sing-box` binary for those configs even when `[runtime].engine` is
`xray`. Setting `[runtime].engine = "sing-box"` for non-Hysteria2 configs
currently returns a clear unsupported-combination error.

---

## disconnect

Stop the active managed proxy runtime.

```bash
xrat disconnect [flags]
```

### Flags

| Flag     | Description              |
| -------- | ------------------------ |
| `--json` | Print the result as JSON |

### Examples

```bash
xrat disconnect
```

### Behavior

1. Sends a runtime-disconnect request to the daemon over local IPC
2. The daemon sends SIGTERM to the running proxy process
3. Waits up to 5 seconds for graceful shutdown
4. Sends SIGKILL if the process is still running
5. Updates the session status to `stopped`
6. Cleans up temporary config files

---

## status

Show the managed proxy runtime status.

```bash
xrat status [flags]
```

### Flags

| Flag     | Description              |
| -------- | ------------------------ |
| `--json` | Print the status as JSON |

### Examples

```bash
xrat status
```

```bash
xrat status --json
```

### Output

Displays:

- **Session state**: `starting`, `running`, `stopping`, `stopped`, `failed`
- **Config details**: protocol, address, port, name
- **Process info**: PID, liveness check
- **Inbound health**: TCP reachability of SOCKS, HTTP, Shadowsocks ports
- **Uptime**: time since session started

If no daemon is reachable, the command exits with a hint to run
`xrat daemon start`.

### JSON Output

```json
{
  "status": "running",
  "session_id": 5,
  "session_config": {
    "ref": "a1b2c3d4e5f6"
  },
  "protocol": "vless",
  "address": "example.com",
  "port": 443,
  "pid": 12345,
  "pid_alive": true,
  "socks_port": 18200,
  "socks_reachable": true,
  "http_port": 18201,
  "http_reachable": false,
  "started_at": "2026-05-28T10:30:00Z"
}
```

## Related

- [`daemon`](daemon.md) — persistent daemon with auto-rotation
- [`proxy`](proxy.md) — control auto-rotation scheduling
- [`test`](test.md) — test configs before connecting
- [`parse`](parse.md) — parse and preview Xray or sing-box runtime JSON
