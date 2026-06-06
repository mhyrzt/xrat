# Config Management Commands

Manage individual stored configs after import.

These commands operate on config IDs from `xrat list configs`.

## When to Use These Commands

| Command   | Use when you want to                                               |
| --------- | ------------------------------------------------------------------ |
| `add`     | Store one share link without creating a subscription source record |
| `show`    | Inspect one stored config or subscription                          |
| `enable`  | Include a config in normal filtered workflows                      |
| `disable` | Keep a config stored but skip it in normal filtered workflows      |
| `delete`  | Hide a config from normal lists, or remove a subscription          |
| `restore` | Bring a soft-deleted config back                                   |
| `purge`   | Permanently remove all soft-deleted configs                        |

## Config State

`active`, `enabled`, and `deleted` are separate states.

| State      | Meaning                                                     |
| ---------- | ----------------------------------------------------------- |
| `active`   | The config used by the current managed runtime session.     |
| `enabled`  | Included in normal list, test, and rotation workflows.      |
| `disabled` | Stored but skipped by enabled-only workflows.               |
| `deleted`  | Soft-deleted and hidden unless `--deleted` or `--all` used. |

Use `xrat connect <id>` when you want to start a proxy runtime. Use
`xrat rotate start` when you want the daemon to manage automatic rotation.

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

Show details for one stored config or subscription. The target is a required
subcommand (`config` or `subscription`).

```bash
xrat show config <id> [--json]
xrat show subscription <id> [--json]
```

### Arguments

| Argument | Description                       |
| -------- | --------------------------------- |
| `id`     | Config or subscription ID to show |

### Flags

| Flag     | Description              |
| -------- | ------------------------ |
| `--json` | Print the result as JSON |

### Examples

```bash
xrat show config 42
xrat show config 42 --json
xrat show subscription 3
```

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

`enable`/`disable` are idempotent: enabling an already-enabled config (or a
deleted one) prints an informational notice and exits successfully without
changing state.

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

Delete a config (soft by default) or a whole subscription. The target is a
required subcommand (`config` or `subscription`).

```bash
xrat delete config <id> [--hard]
xrat delete subscription <id> [--yes]
```

### Arguments

| Argument | Description                         |
| -------- | ----------------------------------- |
| `id`     | Config or subscription ID to delete |

### Flags

| Flag     | Description                                            |
| -------- | ------------------------------------------------------ |
| `--hard` | (config) Permanently delete the config instead of soft |
| `--yes`  | (subscription) Skip the confirmation prompt            |

Soft-deleted configs are hidden from normal lists but can still be viewed with:

```bash
xrat list configs --deleted
xrat list configs --all
```

Use `delete config --hard` only when the row should be permanently removed.
`delete subscription` permanently removes the subscription **and all of its
configs** (plus their test history and runtime sessions), so it prompts for
confirmation unless `--yes` is given.

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
that was removed with `delete config --hard`.

---

## purge

Permanently delete **all** soft-deleted configs in one step, along with their
test history and runtime sessions.

```bash
xrat purge [--yes]
```

### Flags

| Flag    | Description                  |
| ------- | ---------------------------- |
| `--yes` | Skip the confirmation prompt |

`purge` reports how many configs are pending and prompts for confirmation before
deleting. In a non-interactive shell it aborts unless `--yes` is given. This is
irreversible — restore anything you want to keep with `xrat restore` first.

```bash
xrat purge          # prompts: Permanently delete N soft-deleted config(s)? [y/N]
xrat purge --yes    # no prompt
```

## Related

- [`list`](list.md) — find config IDs and filter by state
- [`runtime`](runtime.md) — connect, disconnect, and inspect active sessions
- [`tui`](tui.md) — manage configs interactively
