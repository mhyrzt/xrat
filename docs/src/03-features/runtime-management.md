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
3. **Verify process identity** — Compare the process executable and command line
   (queried via `sysinfo`, so it works across Linux/macOS/BSD) with the expected
   runtime engine and session config
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

### Restore after a graceful daemon shutdown

Set `runtime.restore_connection_on_start = true` to reconnect the last managed
profile when the daemon starts after a graceful shutdown. The saved
`runtime.tun.enabled` setting selects the same TUN or local-proxy mode used by
the connection. A manual `xrat disconnect` clears the restore intent. Startup
retries failed connections with increasing delays while the daemon remains
running, so a network that becomes available after boot can still connect.

This works in the background daemon, including a lingering user service; the
TUI and a graphical login are not required. The option defaults to `false`.

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

## Routing

Managed sessions apply `[routing.direct]` before `[routing.block]`, with the
proxy as the default route. Xray/V2Ray supports domain, IP/CIDR, geosite, and
GeoIP rules. sing-box supports domain and IP/CIDR rules; xrat reports an error
for sing-box geosite/GeoIP entries until rule-set translation is available.
Connection-test probes intentionally remain proxy-only.

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

| Engine     | Binary     | Protocols                                                                          |
| ---------- | ---------- | ---------------------------------------------------------------------------------- |
| `xray`     | `xray`     | All listed protocols, including native Hysteria2 when its fields are representable |
| `v2ray`    | `v2ray`    | VLESS, VMess, Shadowsocks, Trojan, HTTP, SOCKS5                                    |
| `sing-box` | `sing-box` | All listed protocols (VLESS, VMess, Shadowsocks, Trojan, HTTP, SOCKS5, Hysteria2)  |

Every protocol has a managed sing-box generator. `sing-box` sessions and
`xrat test --engine sing-box` require a sing-box `>=1.13.0` binary; newer
versions are accepted and the pinned conformance target is `v1.13.21`. Link
parameters that cannot be represented exactly, unsupported ciphers/transports,
and a non-loopback stats controller are rejected before launch. Xray generation
still rejects unsupported Hy2 URI options, and V2Ray rejects Hysteria2.

## TUN Capture

Xray and sing-box can capture system-wide traffic through a TUN interface so
applications that ignore proxy settings still egress through the active config:

```toml
[runtime]
engine = "xray"  # "xray" | "sing-box"; V2Ray rejects TUN

[runtime.tun]
enabled = true
interface_name = "xrat0"
auto_route = true
```

TUN capture needs elevated network privileges. Grant `CAP_NET_ADMIN`
(and `CAP_NET_RAW`) to the managed binary of the selected engine:

```sh
xrat tun setup
```

The systemd user unit installed by `xrat daemon install` keeps `NoNewPrivileges=true`
by default to maintain standard proxy process hardening. When TUN is enabled,
running `xrat tun setup` or `xrat daemon install --tun` installs a supported drop-in
override `~/.config/systemd/user/xrat-daemon.service.d/10-tun.conf` with
`NoNewPrivileges=false` and reloads `systemctl --user daemon-reload`. Effective
daemon and service readiness can be verified with `xrat tun status`. File capabilities
are lost whenever the managed core is reinstalled or upgraded.

At least one local inbound (SOCKS by default) must stay enabled. Xray emits a
native `protocol: "tun"` inbound and manages routes with `autoSystemRoutingTable`;
this needs a core with working Linux TUN support (Xray >= 26.7.11),
and older cores are rejected before launch. Generated Xray routes match the address
families specified in `[runtime.tun].address` (two `/1` prefixes per family,
or dual-stack). These routes beat a physical default route even at metric zero
without replacing it. Routes are omitted entirely if `auto_route = false`.
sing-box emits `type: "tun"` with `route.auto_detect_interface` and routes
private/LAN destinations direct. V2Ray reports an unsupported error.
Stack selection, `strict_route` and route exclusions are sing-box-only; Xray
rejects non-default requests for these settings. Xray always uses its native
stack. See the [managed capture contract](../06-architecture/managed-capture.md)
and [DNS policies](dns-policies.md) for version limits and verification commands.

### Lifecycle and Interface Safety

macOS managed capture uses sing-box, a root runtime owner and a numbered `utun`
interface. Xray managed TUN remains Linux-only. The kernel releases `utun` when
the engine closes its socket; XRAT verifies release and retains ownership on
failure. Route preflight refuses VPN default routes and capture conflicts.
See [`tun` on macOS](../02-cli/tun.md#macos) for setup and initial limitations.

Engine capabilities and native preflight syntax validation (`xray run -test` / `sing-box check`)
are verified before disconnecting or stopping any active session. If preflight or capability
checks fail, the current healthy session continues running. Preflight runs before the server's
runtime start and does not create the TUN interface.

Leftover TUN interfaces are cleaned up safely prior to starting the engine. xrat tracks
interface ownership in `tun-ownership.json` and verifies kernel sysfs device metadata
(`IFF_TUN`) before deleting any interface via in-process Linux netlink (`RTM_DELLINK`). Foreign
interfaces, non-TUN devices, missing verified interface indices, or interface index
mismatches are strictly rejected. A pending startup record without an index does
not authorize cleanup. Ownership records remain available through process teardown
and are cleared once the interface is gone or verified cleanup succeeds.
Ownership checks and native validation run before replacing an active runtime;
cleanup and startup failures during handoff attempt to restore its previous config.
If safe cleanup or restoration is impossible, the command reports the rollback
failure and preserves unverified interfaces for manual inspection.

### DNS

In TUN mode, Xray intercepts DNS traffic (diverting destination port 53 traffic from `tun-in` to a dedicated `dns-out` outbound), routing DNS queries through configured DoH servers (defaulting to Cloudflare and Google over IP). To break bootstrapping routing loops before the tunnel starts, node endpoints and DNS provider hosts are pre-resolved into static hosts entries. Private and local network ranges (RFC 1918 / RFC 4193) are automatically routed `direct` to keep local resources accessible.

## Related

- [`connect` CLI](../02-cli/runtime.md#connect) — command reference
- [Daemon and IPC](daemon-and-ipc.md) — daemon-managed sessions
- [Auto-Rotation](auto-rotation.md) — automatic proxy switching
- [Config Generation](../06-architecture/config-generation.md) — how configs are
  generated
