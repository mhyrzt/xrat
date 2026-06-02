# geoip

Manage GeoLite2 MMDB assets and inspect GeoIP lookup configuration.

```bash
xrat geoip <command> [flags]
```

## Subcommands

| Command                 | Description                                              |
| ----------------------- | -------------------------------------------------------- |
| [`download`](#download) | Download one or more GeoLite2 MMDB editions              |
| [`update`](#update)     | Refresh all supported GeoLite2 MMDB editions             |
| [`path`](#path)         | Print the resolved MMDB directory                        |
| [`status`](#status)     | Show MMDB presence and size for each supported edition   |
| [`lookup`](#lookup)     | Look up a single IP through the configured GeoIP backend |
| [`backend`](#backend)   | Print the active GeoIP backend configuration             |

---

## download

Download one or more GeoLite2 MMDB editions.

```bash
xrat geoip download [flags]
```

| Flag               | Description                                                                                                      |
| ------------------ | ---------------------------------------------------------------------------------------------------------------- |
| `--edition <name>` | Edition to download. Repeatable: `GeoLite2-Country`, `GeoLite2-City`, `GeoLite2-ASN` or `country`, `city`, `asn` |
| `--all`            | Download all supported editions                                                                                  |
| `--output <dir>`   | Override the MMDB target directory for this command                                                              |
| `--force`          | Re-download even when the destination file already exists                                                        |
| `--url <url>`      | Override the download URL template. Use `{edition}` as a placeholder                                             |
| `--timeout <secs>` | Override the HTTP request timeout in seconds                                                                     |
| `--quiet`          | Suppress progress bar output                                                                                     |

If neither `--edition` nor `--all` is given, the configured `default_editions`
from `[mmdb]` are used.

### Examples

Download all editions to the default MMDB directory:

```bash
xrat geoip download --all
```

Download a single edition:

```bash
xrat geoip download --edition city
```

Download to a custom directory:

```bash
xrat geoip download --all --output ./testdata/xrat/mmdb
```

---

## update

Refresh all supported GeoLite2 MMDB editions. Equivalent to
`download --all --force`.

```bash
xrat geoip update [flags]
```

| Flag               | Description                                                          |
| ------------------ | -------------------------------------------------------------------- |
| `--output <dir>`   | Override the MMDB target directory for this command                  |
| `--url <url>`      | Override the download URL template. Use `{edition}` as a placeholder |
| `--timeout <secs>` | Override the HTTP request timeout in seconds                         |
| `--quiet`          | Suppress progress bar output                                         |

### Example

```bash
xrat geoip update
```

---

## path

Print the resolved MMDB directory.

```bash
xrat geoip path [flags]
```

| Flag             | Description                                         |
| ---------------- | --------------------------------------------------- |
| `--output <dir>` | Override the MMDB target directory for this command |

Resolution order:

1. `--output` flag, if provided
2. `[mmdb].dir` from config (resolved relative to `XRAT_PATH` or config file
   location)
3. Default: `~/.config/xrat/mmdb`

### Examples

```bash
xrat geoip path
xrat geoip path --output /custom/path
```

---

## status

Show MMDB presence and size for each supported edition.

```bash
xrat geoip status [flags]
```

| Flag             | Description                                         |
| ---------------- | --------------------------------------------------- |
| `--output <dir>` | Override the MMDB target directory for this command |
| `--strict`       | Exit non-zero when any supported edition is missing |

### Example

```bash
xrat geoip status --strict
```

---

## lookup

Look up a single IP address through the configured GeoIP backend.

```bash
xrat geoip lookup <ip> [flags]
```

| Argument | Description           |
| -------- | --------------------- |
| `ip`     | IP address to look up |

| Flag               | Description                                               |
| ------------------ | --------------------------------------------------------- |
| `--backend <name>` | Override backend: `mmdb`, `ipwhois`, `ip-api`             |
| `--no-cache`       | Bypass the configured in-memory cache for this invocation |
| `--json`           | Print the lookup result as JSON                           |

The lookup returns country code, city/region, and ASN information when
available.

### Examples

```bash
xrat geoip lookup 8.8.8.8
xrat geoip lookup 8.8.8.8 --backend ipwhois
xrat geoip lookup 2001:4860:4860::8888 --json
```

---

## backend

Print the active GeoIP backend configuration.

```bash
xrat geoip backend [flags]
```

| Flag               | Description                                       |
| ------------------ | ------------------------------------------------- |
| `--backend <name>` | Override backend: `mmdb`, `ipwhois`, `ip-api`     |
| `--no-cache`       | Describe the backend chain without cache wrapping |

Shows the backend type, configured fallback, rate limiting, and cache settings.

### Example

```bash
xrat geoip backend
```

## Related

- [`[mmdb]` config](../05-reference/config-file.md#mmdb) — MMDB asset
  configuration
- [`[testing.geoip]` config](../05-reference/config-file.md#testinggeoip) —
  GeoIP lookup backend configuration
- [GeoIP Enrichment](../03-features/testing.md#geoip-enrichment) — test result
  enrichment feature
