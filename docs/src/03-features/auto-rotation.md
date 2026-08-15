# Auto-Rotation

xrat supports automatic proxy rotation, periodically switching between configs
based on a schedule, health checks, or manual triggers.

## Overview

Auto-rotation is managed by the daemon supervisor. When enabled, the daemon:

1. Periodically tests candidate configs
2. Selects the best candidate based on latency
3. Atomically disconnects the old session and connects the new one
4. Respects cooldown periods to prevent rapid switching

## Configuration

Enable and configure rotation in `config.toml`:

```toml
[runtime.rotation]
enabled = true
interval_secs = 1800
health_trigger_enabled = true
health_failure_threshold = 3
cooldown_secs = 300
test_concurrency = 0
test_stages = ["icmp", "real_delay"]
refresh_subscriptions = false
```

| Field                      | Description                                                | Default                  |
| -------------------------- | ---------------------------------------------------------- | ------------------------ |
| `enabled`                  | Enable scheduled and health-triggered rotation             | `true`                   |
| `interval_secs`            | Rotation interval in seconds                               | `1800` (30 minutes)      |
| `health_trigger_enabled`   | Trigger recovery when the active runtime becomes unhealthy | `true`                   |
| `health_failure_threshold` | Consecutive proxied HTTP failures required for recovery    | `3`                      |
| `cooldown_secs`            | Per-config cooldown after health failure                   | `300` (5 minutes)        |
| `test_concurrency`         | Concurrent test workers (`0` = auto)                       | `0`                      |
| `test_stages`              | Fresh candidate test stages                                | `["icmp", "real_delay"]` |
| `refresh_subscriptions`    | Refresh URL subscriptions before candidate selection       | `false`                  |

### Refresh Before Rotation

With `refresh_subscriptions = true`, automatic (timer/health) rotation first
re-fetches every URL-backed subscription using the same import + reconciliation
path as a manual refresh: still-present configs are updated, and
provider-removed configs are soft-deleted so they are excluded from candidate
selection. Non-URL sources are skipped. Refresh failures are recorded as
separate `subscription`/`rotation` events and never abort rotation — selection
proceeds with whatever configs are present, so the old runtime is not left
stopped because a provider was unreachable.

This applies to timer- and health-triggered rotation. For a one-off manual
rotation, use `xrat rotate now --refresh` instead (see
[`rotate now`](../02-cli/rotate.md#rotate-now)).

## Rotation Triggers

### Timer Trigger

Scheduled rotation every `interval_secs`:

```toml
interval_secs = 1800  # rotate every 30 minutes
```

The timer runs only while a managed runtime is active. Starting the daemon with
no runtime does not select or start a config automatically. A failed timer
attempt is rescheduled for the normal interval instead of retrying in a tight
loop.

### Health Check Trigger

Triggered when the active proxy fails a health check:

```toml
health_trigger_enabled = true
```

The daemon monitors proxy health every 15 seconds. A dead runtime process or an
unreachable configured inbound triggers recovery immediately. When an active
SOCKS5 or HTTP inbound exists, xrat also sends an asynchronous HTTP request
through that proxy using `[testing.real_delay]` URL, timeout, redirect, and
accepted-status settings. These data-plane failures trigger recovery only after
`health_failure_threshold` consecutive failures; a successful probe resets the
counter. Results from a session that has already been replaced are discarded.

Shadowsocks-only runtimes use process and inbound-socket checks because xrat
does not currently run the HTTP probe through a Shadowsocks client.

### Manual Trigger

Triggered by the user via CLI:

```bash
xrat rotate now
```

Unpinned manual rotation bypasses the timer and per-config cooldown, but still
runs fresh candidate tests.

### Forced Rotation

Rotate to a specific config:

```bash
xrat rotate now --config-id 99
```

Skips candidate selection and candidate health ranking. The target must still be
enabled, different from the active config, and pass native engine preflight
validation.

## Cooldown Protection

After a health failure, the failed config receives a per-config cooldown:

```toml
cooldown_secs = 300  # 5 minutes
```

Automatic timer and health selection excludes configs whose cooldown has not
expired. Manual rotation may select them; a pinned manual target explicitly
bypasses cooldown.

## Candidate Selection

When rotating without `--config-id`, the daemon selects the best candidate:

### Step 1: Load Candidates

Query enabled configs from the database:

```sql
SELECT * FROM configs
WHERE is_enabled = true
  AND is_deleted = false
  AND id != <current_config_id>
```

### Step 2: Test Candidates

Run the configured stages freshly on all candidates concurrently. Stored test
rows are not reused for rotation:

```toml
test_concurrency = 4  # test 4 configs at once
test_stages = ["icmp", "real_delay"]
```

For each candidate, xrat runs the normal test pipeline and records the result.
ICMP is diagnostic only. A candidate qualifies by real-delay when that stage
ran, otherwise by download, otherwise by TCP. A candidate cannot qualify from
ICMP alone.

### Step 3: Filter Failures

Exclude configs that do not pass the qualifying stage.

### Step 4: Sort by Latency

Sort by real-delay latency (lowest first):

```rust
successful.sort_by_key(|c| c.real_delay_ms);
```

### Step 5: Select Top Candidate

Pick the first config from the sorted list:

```rust
let best = successful.first()?;
```

If no candidates pass testing, rotation is skipped.

## Rotation Flow

When rotation is triggered:

1. **Select candidate** — Run fresh tests, or validate an explicit config ID
2. **Preflight** — Run the selected engine's native config validator
3. **Handoff** — Stop the old process and start the replacement on the same
   inbounds
4. **Verify** — Wait for the replacement inbound to become reachable
5. **Commit or roll back** — Mark the replacement active, or reconnect the old
   config
6. **Reschedule** — Record the result and schedule the normal next interval

Native preflight commands are `xray run -test -c`, `v2ray test -c`, and
`sing-box check -c`. Preflight happens before disruption. Because the old and
new runtime use the same local ports, the process handoff cannot overlap; if the
replacement fails after the old process stops, xrat reconnects the previous
config and reports both the replacement and rollback outcome.

## Rotation Status

Check rotation status:

```bash
xrat rotate status
```

Output:

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

```bash
xrat rotate status --json
```

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

## Enabling Rotation

### Start Rotation

```bash
xrat rotate enable
```

Atomically writes `runtime.rotation.enabled = true` to `config.toml`, then
enables the running daemon scheduler.

### Stop Rotation

```bash
xrat rotate disable
```

Atomically writes `runtime.rotation.enabled = false` to `config.toml`, then
disables the running daemon scheduler. The active proxy session continues.

## Rotation Strategies

### Conservative Strategy

Long intervals, strict testing, long cooldown:

```toml
[runtime.rotation]
enabled = true
interval_secs = 3600  # 1 hour
health_trigger_enabled = true
cooldown_secs = 600   # 10 minutes
test_stages = ["icmp", "real_delay"]
```

Best for: stable connections, minimal disruption

### Aggressive Strategy

Short intervals, fast testing, short cooldown:

```toml
[runtime.rotation]
enabled = true
interval_secs = 300   # 5 minutes
health_trigger_enabled = true
cooldown_secs = 60    # 1 minute
test_stages = ["real_delay"]
```

Best for: finding the fastest proxy, frequent optimization

### Health-Only Strategy

No scheduled rotation, only rotate on failure:

```toml
[runtime.rotation]
enabled = true
interval_secs = 86400  # 24 hours (effectively disabled)
health_trigger_enabled = true
cooldown_secs = 300
test_stages = ["real_delay"]
```

Best for: stable connections with automatic failover

## Persistence

The enabled setting survives daemon restarts because `rotate enable` and
`rotate disable` update `config.toml`. Runtime counters and scheduling state are
in memory, while per-config cooldown/failure metadata and the active runtime
session are persisted in the database.

## Troubleshooting

### Rotation Not Triggering

**Symptom**: Timer fires but rotation doesn't happen

**Check**:

- Is cooldown active? `xrat rotate status`
- Are there enabled configs? `xrat list configs --enabled-only`
- Do candidates pass testing? `xrat test --enabled-only`

### Rotation Fails

**Symptom**: Rotation triggers but new session fails to connect

**Check**:

- Test the target config manually: `xrat test <id>`
- Check daemon logs for errors
- Verify Xray binary is available

### Rapid Rotation

**Symptom**: Proxy switches too frequently

**Fix**: Increase cooldown:

```toml
cooldown_secs = 600  # 10 minutes
```

Or disable health trigger:

```toml
health_trigger_enabled = false
```

## Related

- [`proxy` CLI](../02-cli/proxy.md) — command reference
- [Daemon and IPC](daemon-and-ipc.md) — daemon supervisor
- [Testing](testing.md) — test stages used for candidate selection
- [Runtime Management](runtime-management.md) — session lifecycle
