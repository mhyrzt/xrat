# rotate

Control automatic proxy rotation scheduling via the daemon.

```bash
xrat rotate <action> [flags]
```

All `rotate` actions require a running daemon:

```bash
xrat daemon start
```

> Rotation scheduling moved here from the old `xrat proxy start|status|stop`
> commands. The `proxy` namespace now covers local proxy endpoints and host/
> session integration; see [proxy](proxy.md).

## Actions

| Action   | Description                                         |
| -------- | --------------------------------------------------- |
| `start`  | Enable automatic proxy rotation on a fixed schedule |
| `stop`   | Disable automatic proxy rotation                    |
| `status` | Show the current proxy rotation status              |
| `now`    | Trigger an immediate manual rotation                |

---

## rotate start

Enable automatic proxy rotation on a fixed schedule.

```bash
xrat rotate start
```

The daemon enables the rotation scheduler using `[runtime.rotation]` settings
from `config.toml` (interval, health trigger, cooldown). Rotation state is
volatile and resets to config defaults on daemon restart.

---

## rotate stop

Disable automatic proxy rotation. The active proxy session keeps running; only
the scheduler is disabled.

```bash
xrat rotate stop
```

---

## rotate status

Show the current rotation status.

```bash
xrat rotate status [--json]
```

### Flags

| Flag     | Description                  |
| -------- | ---------------------------- |
| `--json` | Print rotation status as JSON |

---

## rotate now

Trigger an immediate manual rotation.

```bash
xrat rotate now [--config-id <id-or-ref>] [--refresh]
```

### Flags

| Flag          | Description                                                |
| ------------- | ---------------------------------------------------------- |
| `--config-id` | Force rotation to a specific enabled config ID or ref prefix |
| `--refresh`   | Refresh URL-backed subscriptions before selecting a candidate |

### Behavior

1. If `--refresh` is provided, re-fetches URL-backed subscriptions before
   anything else, so the candidate pass sees the freshest configs.
2. If `--config-id` is provided, rotates to that specific config.
3. Otherwise, selects the best candidate from enabled configs:
   - Tests candidates using `test_stages` from `config.toml`.
   - Picks the config with the lowest real-delay latency.
4. Stops the old session and starts the replacement on the same configured local
   inbound ports.
5. Respects the cooldown period.

## Related

- [`proxy`](proxy.md) — local proxy endpoints, shell, desktop, and PAC helpers
- [`daemon`](daemon.md) — daemon must be running for rotation
- [`connect`](runtime.md#connect) — start one proxy session through the daemon
- [`test`](test.md) — test configs before enabling rotation
