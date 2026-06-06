# proxy

Control auto-rotating proxy scheduling via the daemon.

```bash
xrat proxy <action> [flags]
```

All `proxy` actions require a running daemon:

```bash
xrat daemon start
```

For startup on login, install the systemd user service:

```bash
xrat daemon install --start
```

For startup at boot before login, also enable user lingering:

```bash
loginctl enable-linger $USER
```

## Actions

| Action   | Description                                         |
| -------- | --------------------------------------------------- |
| `start`  | Enable automatic proxy rotation on a fixed schedule |
| `status` | Show the current proxy rotation status              |
| `rotate` | Trigger an immediate manual rotation                |
| `stop`   | Disable automatic proxy rotation                    |

---

## proxy start

Enable automatic proxy rotation on a fixed schedule.

```bash
xrat proxy start
```

### Flags

No command-specific flags.

### Behavior

1. Sends a request to the daemon via IPC
2. Daemon enables the rotation scheduler with settings from `config.toml`:

```toml
[runtime.rotation]
enabled = true
interval_secs = 1800
health_trigger_enabled = true
cooldown_secs = 300
test_concurrency = 0
test_stages = ["real_delay", "download"]
```

3. Daemon begins periodic rotation according to the interval

### Rotation Triggers

| Trigger          | Description                                                           |
| ---------------- | --------------------------------------------------------------------- |
| **Timer**        | Scheduled rotation every `interval_secs`                              |
| **Health check** | Triggered when proxy health check fails (if `health_trigger_enabled`) |
| **Manual**       | Triggered by `xrat proxy rotate`                                      |

### Cooldown

After a rotation, the daemon waits `cooldown_secs` before allowing another
rotation. This prevents rapid switching when multiple triggers fire.

---

## proxy status

Show the current proxy rotation status.

```bash
xrat proxy status [flags]
```

### Flags

| Flag     | Description                   |
| -------- | ----------------------------- |
| `--json` | Print rotation status as JSON |

### Output

```
Proxy Rotation Status
─────────────────────────────────
Enabled:        yes
Interval:       1800s
Last rotation:  2026-05-28 10:30:00 (manual)
Next rotation:  2026-05-28 11:00:00
Cooldown:       300s (inactive)
Active config:  42 (vless://example.com:443)
```

### JSON Output

```json
{
  "enabled": true,
  "interval_secs": 1800,
  "last_trigger": "manual",
  "last_rotation_at": "2026-05-28T10:30:00Z",
  "next_rotation_at": "2026-05-28T11:00:00Z",
  "cooldown_secs": 300,
  "cooldown_active": false,
  "active_config_id": 42
}
```

---

## proxy rotate

Trigger an immediate manual rotation.

```bash
xrat proxy rotate [flags]
```

### Flags

| Flag               | Description                                                   |
| ------------------ | ------------------------------------------------------------- |
| `--config-id <id>` | Force rotation to a specific enabled config ID                |
| `--refresh`        | Refresh URL-backed subscriptions before selecting a candidate |

### Examples

Rotate to the best candidate automatically:

```bash
xrat proxy rotate
```

Rotate to a specific config:

```bash
xrat proxy rotate --config-id 99
```

Refresh subscriptions first, then rotate to the best fresh candidate:

```bash
xrat proxy rotate --refresh
```

### Behavior

1. If `--refresh` is provided, re-fetches URL-backed subscriptions (same import
   - reconciliation path as `xrat import`) before anything else, so the
     candidate pass sees the freshest configs and skips provider-removed ones
2. If `--config-id` is provided, rotates to that specific config
3. Otherwise, selects the best candidate from enabled configs:
   - Tests candidates using `test_stages` from config.toml
   - Picks the config with the lowest real-delay latency
4. Stops the old session and starts the replacement on the same configured local
   inbound ports
5. Respects cooldown period (rotation is delayed if cooldown is active)

For automatic (timer/health) rotation, set
`[runtime.rotation] refresh_subscriptions = true` to perform the same refresh
before each daemon-triggered rotation. Refresh failures are recorded as separate
events and never abort rotation or leave the old runtime stopped.

### Candidate Selection

When rotating without `--config-id`:

1. Loads all enabled configs from the database
2. Excludes the currently active config
3. Tests candidates concurrently (up to `test_concurrency` workers)
4. Filters out configs that fail any test stage
5. Sorts by real-delay latency (lowest first)
6. Selects the top candidate

---

## proxy stop

Disable automatic proxy rotation.

```bash
xrat proxy stop
```

### Flags

No command-specific flags.

### Behavior

1. Sends a request to the daemon via IPC
2. Daemon disables the rotation scheduler
3. Active proxy session continues running (not disconnected)

---

## proxy pac

Work with a Proxy Auto-Config (PAC) file generated from the active runtime
endpoints, so browsers and desktop environments can use per-destination routing
instead of a blunt global proxy.

```bash
xrat proxy pac url     # print the PAC URL served by the API server
xrat proxy pac print   # print the generated PAC file for the active runtime
```

### proxy pac url

Prints the URL the API server serves the PAC file at, for example
`http://127.0.0.1:8787/proxy.pac`. A wildcard bind host (`0.0.0.0`) is shown as
`127.0.0.1`, since PAC consumers should fetch over loopback. If the API server
is disabled, a note explains how to enable it.

### proxy pac print

Generates the PAC file locally from the active runtime's HTTP/SOCKS inbounds and
prints it to stdout. The generated PAC:

- Routes plain hostnames, `*.local`, loopback, and private IP ranges `DIRECT`.
- Prefers SOCKS, then HTTP, then `DIRECT` for everything else.
- With no active runtime, routes everything `DIRECT`.

### PAC route

The PAC file is also served by the Axum API server at:

```
GET /proxy.pac
```

This route is **unauthenticated by default** and returns
`Content-Type: application/x-ns-proxy-autoconfig`. PAC consumers usually cannot
send auth headers, and the file exposes only non-secret local endpoint data
(Shadowsocks credentials are never included). Prefer a loopback server bind for
PAC use.

## Related

- [`daemon`](daemon.md) — daemon must be running for proxy commands to work
- [`connect`](runtime.md#connect) — start one proxy session through the daemon
- [`test`](test.md) — test configs before enabling rotation
- [`serve`](serve.md) — run the API server that hosts `/proxy.pac`
