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
enabled = false
interval_secs = 1800
health_trigger_enabled = true
cooldown_secs = 300
test_concurrency = 0
test_stages = ["icmp", "real_delay"]
refresh_subscriptions = false
```

| Field                    | Description                                       | Default                      |
| ------------------------ | ------------------------------------------------- | ---------------------------- |
| `enabled`                | Enable scheduled rotation                         | `false`                      |
| `interval_secs`          | Rotation interval in seconds                      | `1800` (30 minutes)          |
| `health_trigger_enabled` | Trigger rotation on health check failure          | `true`                       |
| `cooldown_secs`          | Minimum time between rotations                    | `300` (5 minutes)            |
| `test_concurrency`       | Concurrent test workers (`0` = auto)              | `0`                          |
| `test_stages`            | Test stages to run for candidate selection        | `["icmp", "real_delay"]` |
| `refresh_subscriptions`  | Refresh URL subscriptions before candidate select | `false`                      |

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

The daemon maintains a timer that fires at the specified interval, triggering
rotation to the best available candidate.

### Health Check Trigger

Triggered when the active proxy fails a health check:

```toml
health_trigger_enabled = true
```

The daemon monitors proxy health every 15 seconds. If the health check fails
(process dead, port unreachable), rotation is triggered immediately.

### Manual Trigger

Triggered by the user via CLI:

```bash
xrat rotate now
```

Manual rotation bypasses the timer but respects cooldown.

### Forced Rotation

Rotate to a specific config:

```bash
xrat rotate now --config-id 99
```

Skips candidate selection and rotates to the specified config.

## Cooldown Protection

After a rotation, the daemon enforces a cooldown period:

```toml
cooldown_secs = 300  # 5 minutes
```

During cooldown:

- Timer triggers are delayed until cooldown expires
- Health check triggers are suppressed (unless critical)
- Manual triggers are allowed (user override)

### Cooldown State

```rust
struct SupervisorState {
    cooldown_until: Option<DateTime<Utc>>,
    // ...
}
```

When a rotation completes:

```rust
state.cooldown_until = Some(Utc::now() + Duration::from_secs(cooldown_secs));
```

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

Run test stages on all candidates concurrently:

```toml
test_concurrency = 4  # test 4 configs at once
test_stages = ["icmp", "real_delay"]
```

For each candidate:

1. Generate a probe config
2. Spawn a short-lived Xray process
3. Run the specified test stages
4. Collect results (latency, throughput, success/failure)
5. Terminate the probe process

### Step 3: Filter Failures

Exclude configs that failed any test stage:

```rust
let successful = candidates.into_iter()
    .filter(|c| c.real_delay_ok && c.download_ok)
    .collect();
```

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

1. **Check cooldown** — If active, delay or skip
2. **Select candidate** — Run candidate selection (or use `--config-id`)
3. **Atomic replace** — Disconnect old session, connect new session
4. **Update state** — Record rotation timestamp, trigger type, new config ID
5. **Reset timer** — Schedule next timer-based rotation

### Atomic Replace

The replace flow ensures minimal downtime:

```rust
async fn replace_session(old_id: i64, new_config_id: i64) -> Result<()> {
    // 1. Start new session
    let new_session = connect(new_config_id).await?;

    // 2. Wait for new session to be ready
    wait_for_ready(new_session.socks_port).await?;

    // 3. Stop old session
    disconnect(old_id).await?;

    Ok(())
}
```

The new session is started **before** the old session is stopped, ensuring
continuous connectivity.

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

Sends a `ProxyStart` request to the daemon, which enables the rotation
scheduler.

### Stop Rotation

```bash
xrat rotate disable
```

Sends a `ProxyStop` request to the daemon, which disables the rotation
scheduler. The active proxy session continues running.

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

Rotation state is tracked in memory (not persisted to database). On daemon
restart:

- Rotation is disabled (must be re-enabled with `xrat rotate enable`)
- Cooldown is reset
- Timer is reset

The active session is persisted and reattached on daemon restart.

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
