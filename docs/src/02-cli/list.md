# list

List stored configs or subscription sources.

```bash
xrat list <target> [flags]
```

## Targets

| Target          | Alias   | Description                      |
| --------------- | ------- | -------------------------------- |
| `configs`       | `nodes` | List stored proxy configs        |
| `subscriptions` | `subs`  | List stored subscription sources |

---

## list configs

```bash
xrat list configs [flags]
```

### Flags

| Flag                  | Description                                      |
| --------------------- | ------------------------------------------------ |
| `--enabled-only`      | Show only enabled configs                        |
| `--active-only`       | Show only the active config                      |
| `--selected-only`     | Show only the selected config                    |
| `--deleted`           | Show only soft-deleted configs                   |
| `--all`               | Include soft-deleted configs in results          |
| `--subscription <id>` | Show only configs from the given subscription ID |

### Examples

List all configs:

```bash
xrat list configs
```

List only enabled configs from subscription 3:

```bash
xrat list configs --enabled-only --subscription 3
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

| Flag            | Description                                         |
| --------------- | --------------------------------------------------- |
| `--kind <kind>` | Filter by source kind: `url`, `file`, or `raw-text` |

### Examples

List all subscriptions:

```bash
xrat list subscriptions
```

List only URL-based subscriptions:

```bash
xrat list subscriptions --kind url
```
