# Testing

xrat includes a comprehensive testing pipeline that measures connectivity,
latency, and throughput for stored proxy configs.

## Test Stages

The test command runs up to 5 stages in sequence:

| Stage          | Measures                              | Default  | Implementation                   |
| -------------- | ------------------------------------- | -------- | -------------------------------- |
| **ICMP**       | Ping success and latency              | Enabled  | Spawns system `ping` command     |
| **TCP**        | TCP connect success and latency       | Enabled  | Direct TCP socket connection     |
| **Real Delay** | HTTP round-trip latency through proxy | Enabled  | Spawns proxy, makes HTTP request |
| **Download**   | Download throughput through proxy     | Disabled | Downloads file through proxy     |
| **Upload**     | Upload throughput through proxy       | Disabled | POSTs data through proxy         |

### Stage Configuration

Configure stages in `config.toml`:

```toml
[testing]
concurrency = 0  # 0 = auto-detect
order = ["icmp", "real_delay", "download"]
failure_policy = "continue"

[testing.icmp]
enabled = true
timeout = 3000
attempts = 3

[testing.tcp]
enabled = true
timeout = 5000

[testing.real_delay]
enabled = true
url = "https://www.google.com/generate_204"
timeout = 10_000

[testing.download]
enabled = false
url = "https://cachefly.cachefly.net/50mb.test"
timeout = 30_000

[testing.upload]
enabled = false
url = "https://example.com/upload"
timeout = 30_000
```

### Stage Order

The `order` array controls which stages run and in what sequence. Stages not
listed are skipped.

Example: skip ICMP, run only real-delay and download:

```toml
order = ["real_delay", "download"]
```

### Failure Policy

Controls behavior when a stage fails:

| Policy           | Behavior                                     |
| ---------------- | -------------------------------------------- |
| `continue`       | Run all stages regardless of failures        |
| `skip_remaining` | Stop testing this config after first failure |
| `mark_failed`    | Mark config as failed, skip remaining stages |

## ICMP Stage

Measures ICMP ping latency by spawning the system `ping` command.

### Configuration

```toml
[testing.icmp]
enabled = true
timeout = 3000   # ms per attempt
attempts = 3     # number of ping packets
```

### Output

- `icmp_ok` — boolean success
- `icmp_ms` — average latency in milliseconds
- `icmp_attempts` — number of packets sent

### Implementation

xrat spawns `ping -c <attempts> -W <timeout> <address>` and parses stdout for
packet loss and round-trip times.

## TCP Stage

Measures TCP connection latency to the proxy's `address:port`.

### Configuration

```toml
[testing.tcp]
enabled = true
timeout = 5000  # ms
```

### Output

- `tcp_ok` — boolean success
- `tcp_ms` — connection time in milliseconds
- `failure_kind` — failure classification (if failed)

### Failure Classification

TCP failures are classified into categories:

| Category           | Description                      |
| ------------------ | -------------------------------- |
| `DNS`              | DNS resolution failed            |
| `Timeout`          | Connection timed out             |
| `Refused`          | Connection refused (port closed) |
| `Unreachable`      | Network unreachable              |
| `PermissionDenied` | Permission denied                |
| `TLS`              | TLS handshake failed             |
| `Auth`             | Authentication failed            |
| `Process`          | Proxy process failed to start    |
| `Proxy`            | Proxy returned an error          |
| `Unknown`          | Unclassified failure             |

## Real Delay Stage

Measures actual HTTP round-trip latency through the proxy.

### How It Works

1. Generates a temporary Xray probe config with a local SOCKS inbound
2. Spawns a short-lived Xray process
3. Waits for the SOCKS port to become ready
4. Makes an HTTP request through the proxy to the test URL
5. Measures connect time, TTFB, and total round-trip time
6. Terminates the Xray process

### Configuration

```toml
[testing.real_delay]
enabled = true
url = "https://www.google.com/generate_204"
timeout = 10_000  # ms
```

### Output

- `real_delay_ok` — boolean success
- `real_delay_ms` — total round-trip time
- `connect_ms` — TCP connection time
- `ttfb_ms` — time to first byte
- `http_status` — HTTP response status code

### Probe Config

The probe config uses a minimal setup:

```json
{
  "log": { "loglevel": "warning" },
  "inbounds": [
    {
      "tag": "probe-in",
      "port": <random>,
      "listen": "127.0.0.1",
      "protocol": "socks",
      "settings": { "udp": false }
    }
  ],
  "outbounds": [
    {
      "tag": "proxy",
      "protocol": "<node-protocol>",
      "settings": { ... },
      "stream_settings": { ... }
    }
  ]
}
```

## Download Stage

Measures download throughput by downloading a file through the proxy.

### Configuration

```toml
[testing.download]
enabled = false
url = "https://cachefly.cachefly.net/50mb.test"
timeout = 30_000  # ms
```

### Output

- `download_mbps` — throughput in megabits per second

### Implementation

1. Spawns proxy with the config
2. Downloads the file through the proxy
3. Measures bytes transferred and elapsed time
4. Calculates throughput: `(bytes * 8) / (seconds * 1_000_000)`

## Upload Stage

Measures upload throughput by POSTing data through the proxy.

### Configuration

```toml
[testing.upload]
enabled = false
url = "https://example.com/upload"
timeout = 30_000  # ms
```

### Output

- `upload_mbps` — throughput in megabits per second

## Bulk Testing

Test multiple configs concurrently:

```bash
xrat test --enabled-only --concurrency 4
```

### Concurrency

- `0` = auto-detect based on CPU cores
- Positive values set exact worker count

### Progress Bar

Bulk tests display an animated progress bar (unless `--no-progress` is used):

```
Testing configs ━━━━━━━━━━━━━━━━━━━━ 45/150 30% 2m 15s
```

### Output Formats

| Format | Description                                       |
| ------ | ------------------------------------------------- |
| `tsv`  | Tab-separated values (default, terminal-friendly) |
| `csv`  | Comma-separated values (spreadsheet-friendly)     |
| `json` | JSON array with full details                      |

### Sorting

Sort results by:

| Field            | Description                         |
| ---------------- | ----------------------------------- |
| `status`         | Alive first, then by failure reason |
| `icmp`           | Lowest ICMP latency                 |
| `real-delay`     | Lowest real-delay latency           |
| `download-speed` | Highest download throughput         |
| `protocol`       | Protocol name alphabetically        |
| `address`        | Server address alphabetically       |

## Ping Loop

Continuous monitoring mode for a single config:

```bash
xrat test 42 --ping --ping-interval 2000
```

Runs the test repeatedly until Ctrl+C, printing a live summary:

```
Ping loop for config 42 (vless://example.com:443)
─────────────────────────────────────────────────────
#1  ICMP: 15ms  TCP: 12ms  Real Delay: 145ms  ✓
#2  ICMP: 14ms  TCP: 11ms  Real Delay: 142ms  ✓
#3  ICMP: -     TCP: -     Real Delay: -      ✗ timeout
#4  ICMP: 16ms  TCP: 13ms  Real Delay: 148ms  ✓
```

## GeoIP Enrichment

Optionally enrich test results with GeoIP data (country, city, ASN) using
configurable lookup backends.

### Backend Types

| Backend   | Description                                   |
| --------- | --------------------------------------------- |
| `mmdb`    | Local GeoLite2 MMDB files (default)           |
| `ipwhois` | Remote [ipwhois.app](https://ipwhois.app) API |
| `ip-api`  | Remote [ip-api.com](https://ip-api.com) API   |
| `chain`   | Local MMDB with remote fallback               |

### Configuration

```toml
[testing.geoip]
enabled = true
backend = "mmdb"             # mmdb | ipwhois | ip-api | chain
```

#### mmdb backend

Paths for local GeoLite2 MMDB files. Relative paths are resolved from the config
file location, or from `XRAT_PATH` when set.

```toml
[testing.geoip]
country_path = "mmdb/GeoLite2-Country.mmdb"
city_path = "mmdb/GeoLite2-City.mmdb"
asn_path = "mmdb/GeoLite2-ASN.mmdb"
```

Download MMDB files with the [`geoip download`](../02-cli/geoip.md#download)
command.

#### Remote backends (ipwhois / ip-api)

```toml
[testing.geoip.remote]
provider = "ipwhois"         # ipwhois | ip-api
endpoint = ""                # override API endpoint (empty = provider default)
timeout_ms = 5000
api_key = ""                 # provider-specific (if required)
rate_limit_per_minute = 60
```

#### Chain backend

Primary is local MMDB; falls back to a remote service on cache/miss or MMDB
absence.

```toml
[testing.geoip]
backend = "chain"
fallback = "ipwhois"         # ipwhois | ip-api
```

#### Caching

Remote lookups are cached in memory to reduce API calls:

```toml
[testing.geoip.cache]
enabled = true
ttl_secs = 300               # per-entry TTL
max_entries = 1000
```

### Test Result Enrichment

When GeoIP enrichment is enabled, test results include:

- `endpoint_ip` — resolved IP address
- `endpoint_country` — ISO country code (e.g. `NL`)
- `endpoint_city` — city and country (e.g. `Amsterdam/NL`)
- `endpoint_asn` — Autonomous System Number and organization (e.g.
  `AS15169 Google LLC`)

### Related

- [`geoip` CLI](../02-cli/geoip.md) — manage MMDB assets, inspect backends, and
  run ad-hoc IP lookups
- [`[mmdb]` config](../05-reference/config-file.md#mmdb) — MMDB asset
  configuration
- [`[testing.geoip]` config](../05-reference/config-file.md#testinggeoip) — full
  configuration reference

## Test Runs

Tests are grouped into runs for historical analysis:

| Table                  | Purpose                                    |
| ---------------------- | ------------------------------------------ |
| `connection_test_runs` | Groups test results (id, kind, created_at) |
| `connection_tests`     | Individual test results linked to a run    |

View the latest run summary:

```bash
xrat test --latest-run-summary
```

Filter by country or ASN:

```bash
xrat test --latest-run-summary --country US --asn cloudflare
```

## Persistence

All test results are persisted to the database:

| Field                                             | Description                         |
| ------------------------------------------------- | ----------------------------------- |
| `config_id`                                       | Foreign key to configs table        |
| `run_id`                                          | Foreign key to connection_test_runs |
| `icmp_ok`, `icmp_ms`                              | ICMP results                        |
| `tcp_ok`, `tcp_ms`                                | TCP results                         |
| `real_delay_ok`, `real_delay_ms`                  | Real delay results                  |
| `connect_ms`, `ttfb_ms`, `http_status`            | HTTP details                        |
| `download_mbps`, `upload_mbps`                    | Throughput                          |
| `failure_kind`, `failure_reason`                  | Failure details                     |
| `endpoint_ip`, `endpoint_country`, `endpoint_asn` | GeoIP                               |
| `tested_at`                                       | Timestamp                           |

## Related

- [`test` CLI](../02-cli/test.md) — command reference
- [Runtime Management](runtime-management.md) — uses probe configs for testing
- [Database Schema](../05-reference/database-schema.md) — test result tables
