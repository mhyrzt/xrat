# list

List stored configs or subscriptions.

```bash
xrat list <target> [flags]
```

## Targets

| Target          | Alias   | Description               |
| --------------- | ------- | ------------------------- |
| `configs`       | `nodes` | List stored proxy configs |
| `subscriptions` | `subs`  | List stored subscriptions |

---

## list configs

```bash
xrat list configs [flags]
```

### Flags

| Flag                   | Description                                              |
| ---------------------- | -------------------------------------------------------- |
| `--enabled-only`       | Show only enabled configs                                |
| `--active-only`        | Show only the active config                              |
| `--deleted`            | Show only soft-deleted configs                           |
| `--all`                | Include soft-deleted configs in results                  |
| `--subscription <ref>` | Show only configs from the given subscription ref prefix |
| `--format <format>`    | Output format: `table`, `tsv`, `json` (default: `table`) |

### Examples

List all configs:

```bash
xrat list configs
```

List only enabled configs from a subscription ref:

```bash
xrat list configs --enabled-only --subscription f00d
```

List soft-deleted configs:

```bash
xrat list configs --deleted
```

---

## list subscriptions

```bash
xrat list subscriptions [flags]
```

### Flags

| Flag                | Description                                              |
| ------------------- | -------------------------------------------------------- |
| `--kind <kind>`     | Filter by source kind: `url`, `file`, or `raw-text`      |
| `--format <format>` | Output format: `table`, `tsv`, `json` (default: `table`) |

### Examples

List all subscriptions:

```bash
xrat list subscriptions
```

List only URL-based subscriptions:

```bash
xrat list subscriptions --kind url
```

Human tables show short refs first. Use `--format tsv` or `--format json` for
scripts; those formats use stable refs and omit internal numeric database IDs.
The default `table` format is optimized for humans and may change as the CLI
evolves.

Subscription output includes an `updated_at` timestamp in all formats so you can
see the latest refresh/import time at a glance.
