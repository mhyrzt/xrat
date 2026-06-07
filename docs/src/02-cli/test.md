# test

Test connectivity and latency for stored configs.

```bash
xrat test [ref] [flags]
```

## Arguments

| Argument | Description                                           |
| -------- | ----------------------------------------------------- |
| `ref`    | Config ref prefix. Omit to bulk-test matching configs |

## Filter Flags

When testing multiple configs (no ref specified):

| Flag                   | Description                                                 |
| ---------------------- | ----------------------------------------------------------- |
| `--enabled-only`       | Filter: only enabled configs                                |
| `--active-only`        | Filter: only the active config                              |
| `--subscription <ref>` | Filter: only configs from the given subscription ref prefix |

## Stage Skip Flags

| Flag                | Description                                       |
| ------------------- | ------------------------------------------------- |
| `--skip-icmp`       | Skip the ICMP ping stage                          |
| `--skip-tcp`        | Skip the TCP connectivity stage                   |
| `--skip-real-delay` | Skip the real-delay (HTTP round-trip) stage       |
| `--skip-download`   | Skip the download speed stage                     |
| `--skip-upload`     | Skip the upload speed stage (disabled by default) |

## URL Override Flags

| Flag                   | Description                                                |
| ---------------------- | ---------------------------------------------------------- |
| `--test-url <url>`     | Override the URL used for real-delay checks                |
| `--download-url <url>` | Override the URL used for download speed checks            |
| `--upload-url <url>`   | Enable upload speed stage and set the HTTP POST target URL |

## Timeout Override Flags

| Flag                        | Description                                              |
| --------------------------- | -------------------------------------------------------- |
| `--icmp-timeout <ms>`       | Override ICMP timeout in milliseconds                    |
| `--tcp-timeout <ms>`        | Override TCP connect timeout in milliseconds             |
| `--real-delay-timeout <ms>` | Override real-delay HTTP request timeout in milliseconds |
| `--download-timeout <ms>`   | Override download speed request timeout in milliseconds  |
| `--upload-timeout <ms>`     | Override upload speed request timeout in milliseconds    |

## Concurrency and Output Flags

| Flag                | Description                                                                                             |
| ------------------- | ------------------------------------------------------------------------------------------------------- |
| `--concurrency <n>` | Bulk-test concurrency. `0` = auto-detect                                                                |
| `--format <format>` | Output format: `table`, `tsv`, `csv`, `json` (default: `table`)                                         |
| `--output <file>`   | Write bulk results to a file instead of stdout                                                          |
| `--sort-by <field>` | Sort order: `status`, `icmp`, `real-delay`, `download-speed`, `protocol`, `address` (default: `status`) |
| `--no-progress`     | Hide the animated progress bar                                                                          |

## Ping Loop Flags

| Flag                   | Description                                                        |
| ---------------------- | ------------------------------------------------------------------ |
| `--ping`               | Continuously ping one config until Ctrl+C, printing a live summary |
| `--ping-interval <ms>` | Interval between ping-loop iterations (default: `1000`)            |

## Historical Summary Flags

| Flag                   | Description                                                              |
| ---------------------- | ------------------------------------------------------------------------ |
| `--latest-run-summary` | Print a summary of the latest persisted test run and exit                |
| `--country <iso>`      | Filter latest-run summary by endpoint country ISO code (e.g. `US`, `DE`) |
| `--asn <filter>`       | Filter latest-run summary by ASN (case-insensitive substring match)      |

## Test Stages

The test command can record up to 5 probe result types:

| Stage          | Measures                              | Default  |
| -------------- | ------------------------------------- | -------- |
| **ICMP**       | ICMP ping success and latency         | Enabled  |
| **TCP**        | TCP connect success and latency       | Enabled  |
| **Real Delay** | HTTP round-trip latency through proxy | Enabled  |
| **Download**   | Download throughput through proxy     | Disabled |
| **Upload**     | Upload throughput through proxy       | Disabled |

### Stage Order

The default order for ICMP, real-delay, and download is configurable via
`config.toml`:

```toml
[testing]
order = ["icmp", "real_delay", "download"]
```

TCP is used as a gate before real-delay when enabled. Upload runs after download
only when `--upload-url <url>` is provided; there is no `[testing.upload]`
config section.

### Failure Policy

Controls behavior when a stage fails:

```toml
[testing]
failure_policy = "continue"  # "continue" | "skip_remaining" | "mark_failed"
```

## Examples

Test a single config:

```bash
xrat test a1b2
```

Bulk-test all enabled configs with 4 workers:

```bash
xrat test --enabled-only --concurrency 4
```

Test with custom URLs and timeouts:

```bash
xrat test a1b2 \
  --test-url https://example.com/generate_204 \
  --download-url https://example.com/10mb.test \
  --real-delay-timeout 5000 \
  --download-timeout 15000
```

Skip ICMP and download stages:

```bash
xrat test a1b2 --skip-icmp --skip-download
```

Export results to CSV:

```bash
xrat test --enabled-only --format csv --output results.csv
```

Continuous ping loop:

```bash
xrat test a1b2 --ping --ping-interval 2000
```

View latest test run summary:

```bash
xrat test --latest-run-summary
```

Filter by country and ASN:

```bash
xrat test --latest-run-summary --country US --asn cloudflare
```

## Output Formats

### Table (default)

Aligned human-readable table for terminal use.

### TSV

Tab-separated values for scripts:

```
ref	name	protocol	address	port	icmp_ms	real_delay_ms	download_mbps	upload_mbps	status	error
a1b2c3d4	Primary	vless	example.com	443	15	145			ok
f00d1234	Edge	vmess	edge.com	8443					failed	timeout
```

### CSV

Comma-separated values, spreadsheet compatible.

### JSON

Machine-parseable JSON array with full test result details.

## Failure Classification

Test failures are classified into categories:

| Category           | Description                     |
| ------------------ | ------------------------------- |
| `DNS`              | DNS resolution failed           |
| `Timeout`          | Connection or request timed out |
| `Refused`          | Connection refused              |
| `Unreachable`      | Network unreachable             |
| `PermissionDenied` | Permission denied               |
| `TLS`              | TLS handshake failed            |
| `Auth`             | Authentication failed           |
| `Process`          | Proxy process failed to start   |
| `Proxy`            | Proxy returned an error         |
| `Unknown`          | Unclassified failure            |

## Related

- [`list configs`](list.md#list-configs) — view configs before testing
- [`connect`](runtime.md#connect) — start a proxy for a tested config
