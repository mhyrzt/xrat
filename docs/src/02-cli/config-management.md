# Config Management Commands

Manage individual stored configs after import.

These commands operate on config IDs from `xrat list configs`.

## When to Use These Commands

| Command   | Use when you want to                                               |
| --------- | ------------------------------------------------------------------ |
| `add`     | Store one share link without creating a subscription source record |
| `show`    | Inspect one stored config                                          |
| `select`  | Mark one config as the preferred config for interactive workflows  |
| `enable`  | Include a config in normal filtered workflows                      |
| `disable` | Keep a config stored but skip it in normal filtered workflows      |
| `delete`  | Hide a config from normal lists while preserving history           |
| `restore` | Bring a soft-deleted config back                                   |

## Config State

`selected`, `active`, `enabled`, and `deleted` are separate states.

| State      | Meaning                                                     |
| ---------- | ----------------------------------------------------------- |
| `selected` | The preferred config. This does not start a proxy process.  |
| `active`   | The config used by the current managed runtime session.     |
| `enabled`  | Included in normal list, test, and rotation workflows.      |
| `disabled` | Stored but skipped by enabled-only workflows.               |
| `deleted`  | Soft-deleted and hidden unless `--deleted` or `--all` used. |

Use `xrat connect <id>` when you want to start a proxy runtime. Use
`xrat proxy start` when you want the daemon to manage automatic rotation.

---

## add

Add a single config URI directly to the database.

```bash
xrat add <input>
```

### Arguments

| Argument | Description                                                                                   |
| -------- | --------------------------------------------------------------------------------------------- |
| `input`  | Config URI: `vless://...`, `vmess://...`, `ss://...`, `trojan://...`, `hysteria2://...`, etc. |

### Examples

```bash
xrat add "vless://uuid@example.com:443?type=ws&security=tls#Node"
```

Unlike `xrat import`, `xrat add` does not create or update a subscription source
record.

---

## show

Show details for one stored config.

```bash
xrat show <id> [flags]
```

### Arguments

| Argument | Description       |
| -------- | ----------------- |
| `id`     | Config ID to show |

### Flags

| Flag     | Description              |
| -------- | ------------------------ |
| `--json` | Print the result as JSON |

### Examples

```bash
xrat show 42
xrat show 42 --json
```

---

## select

Select a config as the current selection.

```bash
xrat select <id>
```

### Arguments

| Argument | Description         |
| -------- | ------------------- |
| `id`     | Config ID to select |

Selection is useful for workflows that need a preferred config, including the
TUI. It does not start or stop the managed runtime.

---

## enable

Enable a config.

```bash
xrat enable <id>
```

### Arguments

| Argument | Description         |
| -------- | ------------------- |
| `id`     | Config ID to enable |

Enabled configs are included in normal enabled-only workflows, such as:

```bash
xrat list configs --enabled-only
xrat test --enabled-only
```

---

## disable

Disable a config.

```bash
xrat disable <id>
```

### Arguments

| Argument | Description          |
| -------- | -------------------- |
| `id`     | Config ID to disable |

Disabled configs remain in the database but are excluded from enabled-only
queries, tests, and rotation candidate selection.

---

## delete

Soft-delete a config by default.

```bash
xrat delete <id> [flags]
```

### Arguments

| Argument | Description         |
| -------- | ------------------- |
| `id`     | Config ID to delete |

### Flags

| Flag     | Description                   |
| -------- | ----------------------------- |
| `--hard` | Permanently delete the config |

Soft-deleted configs are hidden from normal lists but can still be viewed with:

```bash
xrat list configs --deleted
xrat list configs --all
```

Use `--hard` only when the row should be permanently removed.

---

## restore

Restore a soft-deleted config.

```bash
xrat restore <id>
```

### Arguments

| Argument | Description          |
| -------- | -------------------- |
| `id`     | Config ID to restore |

`restore` only applies to soft-deleted configs. It does not recreate a config
that was removed with `delete --hard`.

## Related

- [`list`](list.md) — find config IDs and filter by state
- [`runtime`](runtime.md) — connect, disconnect, and inspect active sessions
- [`tui`](tui.md) — manage configs interactively
