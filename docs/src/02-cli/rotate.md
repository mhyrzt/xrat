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

| Action    | Description                                         |
| --------- | --------------------------------------------------- |
| `enable`  | Enable automatic proxy rotation on a fixed schedule |
| `disable` | Disable automatic proxy rotation                    |
| `status`  | Show the current proxy rotation status              |
| `now`     | Trigger an immediate manual rotation                |

---

## rotate enable

Enable automatic proxy rotation on a fixed schedule.

```bash
xrat rotate enable
```

The daemon enables the rotation scheduler using `[runtime.rotation]` settings
from `config.toml` (interval, health trigger, threshold, and cooldown). The
command also writes `runtime.rotation.enabled = true` atomically, so the choice
survives daemon restarts.

---

## rotate disable

Disable automatic proxy rotation. The active proxy session keeps running; only
the scheduler is disabled.

```bash
xrat rotate disable
```

The command writes `runtime.rotation.enabled = false` atomically. It does not
disconnect the active runtime.

---

## rotate status

Show the current rotation status.

```bash
xrat rotate status [--json]
```

### Flags

| Flag     | Description                   |
| -------- | ----------------------------- |
| `--json` | Print rotation status as JSON |

---

## rotate now

Trigger an immediate manual rotation.

```bash
xrat rotate now [--config-id <ref>] [--refresh]
```

### Flags

| Flag          | Description                                                   |
| ------------- | ------------------------------------------------------------- |
| `--config-id` | Force rotation to a specific enabled config ref prefix        |
| `--refresh`   | Refresh URL-backed subscriptions before selecting a candidate |

### Behavior

1. If `--refresh` is provided, re-fetches URL-backed subscriptions before
   anything else, so the candidate pass sees the freshest configs.
2. If `--config-id` is provided, rotates to that specific config.
3. Otherwise, selects the best candidate from enabled configs:
   - Runs fresh tests using `test_stages` from `config.toml`; stored results are
     not reused.
   - Uses real-delay as the qualifying metric when run, then download, then TCP.
     ICMP is diagnostic and cannot qualify a candidate alone.
4. Runs the selected engine's native config validator before stopping the old
   session.
5. Starts the replacement on the same configured local inbound ports. If that
   post-stop handoff fails, xrat attempts to restore the previous runtime.

An explicit `--config-id` bypasses candidate testing and cooldown, but the
config must be enabled, different from the active config, and pass native
preflight validation. An unpinned manual rotation bypasses cooldown but still
runs fresh candidate tests.

## Related

- [`proxy`](proxy.md) — local proxy endpoints, shell, desktop, and PAC helpers
- [`daemon`](daemon.md) — daemon must be running for rotation
- [`connect`](runtime.md#connect) — start one proxy session through the daemon
- [`test`](test.md) — test configs before enabling rotation
