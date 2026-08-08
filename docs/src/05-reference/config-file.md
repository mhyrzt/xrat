# Config File

Full reference for the `config.toml` file with all fields, defaults, and
accepted values.

## File Location

Default: `~/.config/xrat/config.toml`

Resolution order:

1. `--config <path>` CLI flag
2. `XRAT_PATH/config.toml` environment variable
3. `~/.config/xrat/config.toml`

## Top-Level Structure

```toml
[paths]
[database]
[server]
[runtime]
[routing]
[geo]
[parser]
[dns]
[testing]
```

---

## [paths]

Binary paths for proxy engines. All fields are optional (defaults to `$PATH`).

```toml
[paths]
# Database file path (deprecated, use [database.sqlite].path)
database = "db.sqlite"

# Binary paths (optional, defaults to PATH lookup)
xray = "/usr/local/bin/xray"
v2ray = "/usr/local/bin/v2ray"
sing_box = "/usr/local/bin/sing-box"
```

| Field      | Type   | Default    | Description                                              |
| ---------- | ------ | ---------- | -------------------------------------------------------- |
| `database` | string | -          | Database path (deprecated, use `[database.sqlite].path`) |
| `xray`     | string | `xray`     | Xray-core binary path                                    |
| `v2ray`    | string | `v2ray`    | V2Ray binary path                                        |
| `sing_box` | string | `sing-box` | sing-box binary path                                     |

---

## [database]

Database backend selection and connection settings.

```toml
[database]
backend = "sqlite"  # "sqlite" | "postgres"

[database.sqlite]
path = "db.sqlite"

[database.postgres]
user = { env = "XRAT_POSTGRES_USER" }
password = { env = "XRAT_POSTGRES_PASSWORD" }
host = "localhost"
port = 5432
db_name = "xrat"
max_connections = 10
min_connections = 1
connect_timeout_secs = 10
```

| Field                             | Type       | Default     | Description               |
| --------------------------------- | ---------- | ----------- | ------------------------- |
| `backend`                         | enum       | `sqlite`    | `sqlite` or `postgres`    |
| `[sqlite].path`                   | string     | `db.sqlite` | SQLite database file path |
| `[postgres].user`                 | string/env | -           | PostgreSQL username       |
| `[postgres].password`             | string/env | -           | PostgreSQL password       |
| `[postgres].host`                 | string     | `localhost` | PostgreSQL host           |
| `[postgres].port`                 | integer    | `5432`      | PostgreSQL port           |
| `[postgres].db_name`              | string     | -           | PostgreSQL database name  |
| `[postgres].max_connections`      | integer    | `10`        | Connection pool max size  |
| `[postgres].min_connections`      | integer    | `1`         | Connection pool min size  |
| `[postgres].connect_timeout_secs` | integer    | `10`        | Connection timeout        |

---

## [server]

HTTP API server configuration.

```toml
[server]
enabled = false
host = "127.0.0.1"
port = 18203
key = { env = "XRAT_API_KEY" }
pac_enabled = true
pac_allowed_hosts = ["localhost", "127.0.0.1", "::1"]
```

| Field               | Type       | Default                             | Description                             |
| ------------------- | ---------- | ----------------------------------- | --------------------------------------- |
| `enabled`           | boolean    | `false`                             | Enable daemon-hosted API                |
| `host`              | string     | `127.0.0.1`                         | Bind host                               |
| `port`              | integer    | `18203`                             | Bind port                               |
| `key`               | string/env | -                                   | API key for authenticated routes        |
| `pac_enabled`       | boolean    | `true`                              | Serve `/proxy.pac`                      |
| `pac_allowed_hosts` | string[]   | `["localhost", "127.0.0.1", "::1"]` | Allowed `Host` headers for `/proxy.pac` |

`/proxy.pac` is unauthenticated because many PAC consumers cannot send auth
headers. Keep `host = "127.0.0.1"` for local use. If you bind the server to
`0.0.0.0`, add only trusted local DNS names to `pac_allowed_hosts`.

---

## [runtime]

Runtime engine and proxy process configuration.

```toml
[runtime]
engine = "xray"     # "xray" | "v2ray" | "sing-box"
replace_active_session = true
```

| Field                    | Type    | Default | Description                                                                                                                       |
| ------------------------ | ------- | ------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `engine`                 | enum    | `xray`  | Managed runtime engine. Hy2 configs auto-select sing-box; non-Hy2 configs use Xray/V2Ray unless supported by the selected engine. |
| `replace_active_session` | boolean | `true`  | Auto-disconnect on new connect                                                                                                    |

---

### [runtime.rotation]

Proxy auto-rotation settings.

```toml
[runtime.rotation]
enabled = false
interval_secs = 1800
health_trigger_enabled = true
cooldown_secs = 300
test_concurrency = 0
test_stages = ["icmp", "real_delay"]
```

| Field                    | Type     | Default                  | Description                        |
| ------------------------ | -------- | ------------------------ | ---------------------------------- |
| `enabled`                | boolean  | `false`                  | Enable scheduled rotation          |
| `interval_secs`          | integer  | `1800`                   | Rotation interval in seconds       |
| `health_trigger_enabled` | boolean  | `true`                   | Trigger rotation on health failure |
| `cooldown_secs`          | integer  | `300`                    | Minimum time between rotations     |
| `test_concurrency`       | integer  | `0`                      | Test workers (0 = auto)            |
| `test_stages`            | string[] | `["icmp", "real_delay"]` | Candidate test stages              |

---

### [runtime.log]

Proxy process logging.

```toml
[runtime.log]
enabled = true
mask = "none"     # "quarter" | "half" | "full" | "none"
dir = "logs"
dns_log = false
level = "warning" # "debug" | "info" | "warning" | "error"
keep = true
```

| Field     | Type    | Default   | Description                  |
| --------- | ------- | --------- | ---------------------------- |
| `enabled` | boolean | `true`    | Enable logging to files      |
| `mask`    | enum    | `none`    | IP address masking           |
| `dir`     | string  | `logs`    | Log directory                |
| `dns_log` | boolean | `false`   | Enable DNS query logging     |
| `level`   | enum    | `warning` | Log level                    |
| `keep`    | boolean | `true`    | Keep logs after session stop |

---

### [runtime.socks]

SOCKS5 inbound configuration.

```toml
[runtime.socks]
enabled = true
host = "0.0.0.0"
port = 18200
udp = true
auth = { enabled = true, username = "xrat", password = { env = "XRAT_SOCKS_PASSWORD" } }
```

| Field           | Type       | Default   | Description           |
| --------------- | ---------- | --------- | --------------------- |
| `enabled`       | boolean    | `true`    | Enable SOCKS inbound  |
| `host`          | string     | `0.0.0.0` | Bind address          |
| `port`          | integer    | `18200`   | Bind port             |
| `udp`           | boolean    | `true`    | Enable UDP support    |
| `auth.enabled`  | boolean    | `false`   | Enable authentication |
| `auth.username` | string     | `xrat`    | SOCKS username        |
| `auth.password` | string/env | -         | SOCKS password        |

---

### [runtime.http]

HTTP proxy inbound configuration.

```toml
[runtime.http]
enabled = false
host = "0.0.0.0"
port = 18201
```

| Field     | Type    | Default   | Description         |
| --------- | ------- | --------- | ------------------- |
| `enabled` | boolean | `false`   | Enable HTTP inbound |
| `host`    | string  | `0.0.0.0` | Bind address        |
| `port`    | integer | `18201`   | Bind port           |

---

### [runtime.shadowsocks]

Shadowsocks inbound configuration.

```toml
[runtime.shadowsocks]
enabled = false
host = "0.0.0.0"
port = 18202
method = "aes-128-gcm"
password = { env = "XRAT_SHADOWSOCKS_PASSWORD" }
network = "tcp,udp"
```

| Field      | Type       | Default       | Description                |
| ---------- | ---------- | ------------- | -------------------------- |
| `enabled`  | boolean    | `false`       | Enable Shadowsocks inbound |
| `host`     | string     | `0.0.0.0`     | Bind address               |
| `port`     | integer    | `18202`       | Bind port                  |
| `method`   | string     | `aes-128-gcm` | Encryption method          |
| `password` | string/env | -             | Shadowsocks password       |
| `network`  | string     | `tcp,udp`     | Network type               |

---

### [runtime.sniffing]

Traffic sniffing configuration.

```toml
[runtime.sniffing]
enabled = true
dest_override = ["http", "tls", "quic"]
route_only = true
metadata_only = false
domains_excluded = []
ips_excluded = []
```

| Field              | Type     | Default                   | Description                        |
| ------------------ | -------- | ------------------------- | ---------------------------------- |
| `enabled`          | boolean  | `true`                    | Enable traffic sniffing            |
| `dest_override`    | string[] | `["http", "tls", "quic"]` | Protocols for destination override |
| `route_only`       | boolean  | `true`                    | Only sniff for routing             |
| `metadata_only`    | boolean  | `false`                   | Only sniff metadata                |
| `domains_excluded` | string[] | `[]`                      | Excluded domains                   |
| `ips_excluded`     | string[] | `[]`                      | Excluded IPs                       |

### [runtime.stats]

Traffic-stats endpoint exposed by the managed runtime and sampled by the TUI
stats tab. For xray/v2ray this enables the gRPC `StatsService` behind an `api`
inbound; for managed sing-box it binds the Clash API controller
(`experimental.clash_api`). Both bind an extra localhost port, gated by
`enabled`. Probe and stats-disabled runtime configs are unchanged.

```toml
[runtime.stats]
enabled = true
host = "127.0.0.1"
port = 10085
```

| Field     | Type    | Default       | Description                                   |
| --------- | ------- | ------------- | --------------------------------------------- |
| `enabled` | boolean | `true`        | Enable the stats endpoint and TUI stats poller |
| `host`    | string  | `"127.0.0.1"` | Listen host for the stats controller          |
| `port`    | integer | `10085`       | Listen port for the stats controller          |

### [runtime.mux]

Client-side Mux (multiplexing) for generated Xray outbounds, applied to the
proxy outbound of both runtime and probe configs. Disabled by default: Mux
reduces TCP handshakes but commonly hurts throughput (downloads, video, speed
tests), so enable it only for workloads dominated by many short-lived requests.

```toml
[runtime.mux]
enabled = false
concurrency = 8
xudp_concurrency = 0
xudp_proxy_udp443 = "reject"
```

| Field               | Type    | Default    | Description                                                                 |
| ------------------- | ------- | ---------- | --------------------------------------------------------------------------- |
| `enabled`           | boolean | `false`    | Emit a `mux` object on the proxy outbound                                    |
| `concurrency`       | integer | `8`        | Logical connections per Mux session. `0` = Xray default (8); `1..=128`; `-1` disables TCP Mux |
| `xudp_concurrency`  | integer | `0`        | XUDP aggregation concurrency. `0` = legacy path; `1..=1024`; `-1` opts UDP out of Mux |
| `xudp_proxy_udp443` | string  | `"reject"` | QUIC/UDP 443 handling under XUDP: `reject`, `allow`, or `skip`               |

### [runtime.fragment]

TCP fragmentation for generated Xray outbounds. When enabled, the proxy outbound
is chained through a `freedom` outbound (`sockopt.dialerProxy`) that splits early
outgoing TCP writes (typically the TLS ClientHello). This is a
network-circumvention feature whose effect depends on network, transport, and
destination — it can help against some SNI-based filtering but may also hurt.
Disabled by default.

```toml
[runtime.fragment]
enabled = false
packets_mode = "tlshello"
packets = [1, 3]
length = [100, 200]
interval = [10, 20]
```

| Field          | Type      | Default        | Description                                                              |
| -------------- | --------- | -------------- | ----------------------------------------------------------------------- |
| `enabled`      | boolean   | `false`        | Emit the `freedom` fragment outbound and chain the proxy through it     |
| `packets_mode` | string    | `"tlshello"`   | `"tlshello"` (fragment the TLS ClientHello) or `"range"` (use `packets`) |
| `packets`      | integer[] | `[1, 3]`       | Write range `[min, max]` (min ≥ 1, min ≤ max). Used only in `range` mode |
| `length`       | integer[] | `[100, 200]`   | Byte length range `[min, max]` (min ≥ 1, min ≤ max)                     |
| `interval`     | integer[] | `[10, 20]`     | Millisecond delay range `[min, max]` (min ≤ max)                        |

### [runtime.network]

Interface and source binding for managed runtime traffic.

```toml
[runtime.network]
interface = ""
bind_address = ""
mark = 0
listen_interface = ""
```

| Field              | Type    | Default | Description                                                                                       |
| ------------------ | ------- | ------- | ------------------------------------------------------------------------------------------------ |
| `interface`        | string  | `""`    | Outbound interface to bind egress to (Xray `sockopt.interface`, `SO_BINDTODEVICE` on Linux)       |
| `bind_address`     | string  | `""`    | Outbound source IP. **The Xray engine cannot bind a source address and ignores this** (a warning is logged); validated for shape only |
| `mark`             | integer | `0`     | fwmark applied to outbound sockets (Xray `sockopt.mark`). `0` = unset                              |
| `listen_interface` | string  | `""`    | Bind managed inbounds (socks/http/shadowsocks) to this interface's address instead of their host  |

> Interface binding (`interface`, `mark`) and `listen_interface` are
> Linux-focused. `interface` requires a real device name; `listen_interface`
> must resolve to a bindable address or the runtime fails to launch. System-wide
> TUN capture is tracked separately and not provided here.

---

## [routing]

Routing configuration.

```toml
[routing]
domain_strategy = "IPIfNonMatch" # "AsIs" | "IPIfNonMatch" | "IPOnDemand"

[routing.direct]
domain = []
ip = []
geosite = []
geoip = []

[routing.block]
domain = []
ip = []
geosite = []
geoip = []
```

| Field              | Type     | Default        | Description                     |
| ------------------ | -------- | -------------- | ------------------------------- |
| `domain_strategy`  | enum     | `IPIfNonMatch` | Xray domain resolution strategy |
| `[direct].domain`  | string[] | `[]`           | Direct-route domains            |
| `[direct].ip`      | string[] | `[]`           | Direct-route IPs                |
| `[direct].geosite` | string[] | `[]`           | Direct-route geosite categories |
| `[direct].geoip`   | string[] | `[]`           | Direct-route geoip categories   |
| `[block].domain`   | string[] | `[]`           | Blocked domains                 |
| `[block].ip`       | string[] | `[]`           | Blocked IPs                     |
| `[block].geosite`  | string[] | `[]`           | Blocked geosite categories      |
| `[block].geoip`    | string[] | `[]`           | Blocked geoip categories        |

The generated PAC file inlines only curated `domain` entries and IPv4 CIDRs from
`ip` lists. `geosite` and `geoip` lists stay in the proxy engine config and are
not expanded into PAC.

---

## [geo]

GeoIP/geosite asset management.

```toml
[geo]
auto_update = false
update_interval_hours = 168

[[geo.profiles]]
name = "default"
geosite = "https://example.com/geosite.dat"
geoip = "https://example.com/geoip.dat"

[[geo.profiles]]
name = "local"
geosite = "geo/local/geosite.dat"
geoip = "geo/local/geoip.dat"
```

| Field                   | Type    | Default | Description                       |
| ----------------------- | ------- | ------- | --------------------------------- |
| `auto_update`           | boolean | `false` | Enable periodic geo asset updates |
| `update_interval_hours` | integer | `168`   | Update interval in hours          |
| `[[profiles]].name`     | string  | -       | Profile name                      |
| `[[profiles]].geosite`  | string  | -       | Geosite file path or URL          |
| `[[profiles]].geoip`    | string  | -       | GeoIP file path or URL            |

---

## [parser]

Xray JSON schema validation mode.

```toml
[parser]
parse_mode = "strict" # "strict" | "lenient" | "auto"
```

| Field        | Type | Default  | Description               |
| ------------ | ---- | -------- | ------------------------- |
| `parse_mode` | enum | `strict` | Xray JSON validation mode |

---

## [dns]

DNS configuration for generated Xray configs.

```toml
[dns]
query_strategy = "UseSystem" # "UseIP" | "UseIPv4" | "UseIPv6" | "UseSystem"
servers = [
    "8.8.8.8",
    "https://1.1.1.1/dns-query",
]
use_system_hosts = true
disable_cache = false
disable_fallback = false
enable_parallel_query = true

[dns.hosts]
"domain:example.test" = "127.0.0.1"
"domain:lan.test" = ["192.168.1.10", "192.168.1.11"]
```

| Field                   | Type     | Default     | Description             |
| ----------------------- | -------- | ----------- | ----------------------- |
| `query_strategy`        | enum     | `UseSystem` | DNS query strategy      |
| `servers`               | string[] | -           | DNS server list         |
| `use_system_hosts`      | boolean  | `true`      | Use system hosts file   |
| `disable_cache`         | boolean  | `false`     | Disable DNS cache       |
| `disable_fallback`      | boolean  | `false`     | Disable fallback DNS    |
| `enable_parallel_query` | boolean  | `true`      | Enable parallel queries |
| `[hosts]`               | map      | -           | Static DNS entries      |

---

## [mmdb]

Dedicated MaxMind MMDB asset configuration, separate from `[geo]` routing
assets.

```toml
[mmdb]
dir = "mmdb"
download_url = "https://github.com/P3TERX/GeoLite.mmdb/releases/latest/download/{edition}.mmdb"
timeout_secs = 60
default_editions = ["country", "city", "asn"]
auto_update = false
update_interval_hours = 168
```

| Field                   | Type     | Default                                                                          | Description                                                      |
| ----------------------- | -------- | -------------------------------------------------------------------------------- | ---------------------------------------------------------------- |
| `dir`                   | string   | `mmdb`                                                                           | MMDB directory (absolute, or relative to the xrat runtime root)  |
| `download_url`          | string   | `https://github.com/P3TERX/GeoLite.mmdb/releases/latest/download/{edition}.mmdb` | Download URL template. `{edition}` is replaced with edition name |
| `timeout_secs`          | integer  | `60`                                                                             | HTTP request timeout for downloads                               |
| `default_editions`      | string[] | `["country", "city", "asn"]`                                                     | Editions downloaded when no `--edition` or `--all` flag given    |
| `auto_update`           | boolean  | `false`                                                                          | Enable periodic update checks                                    |
| `update_interval_hours` | integer  | `168`                                                                            | Update interval in hours                                         |

The `dir` field is resolved relative to the xrat runtime root (`XRAT_PATH` when
set, otherwise the default app root). Absolute paths are used as-is. The default
per-edition MMDB paths under `[testing.geoip]` also resolve through this MMDB
directory; custom relative per-edition paths are resolved relative to the config
file directory.

---

## [testing]

Connection testing configuration.

```toml
[testing]
concurrency = 0  # 0 = auto
order = ["icmp", "real_delay", "download"]
failure_policy = "continue"  # "continue" | "skip_remaining" | "mark_failed"

[testing.real_delay]
enabled = true
url = "https://www.gstatic.com/generate_204"
timeout = 10_000
# Omit both acceptance fields to accept 200-299.
# accepted_status_codes = [200, 204]
# accepted_status_ranges = ["300-399"]
follow_redirects = true

[testing.icmp]
enabled = true
timeout = 3000
attempts = 3

[testing.download]
enabled = false
url = "https://cachefly.cachefly.net/50mb.test"
timeout = 30_000

[testing.tcp]
enabled = true
timeout = 5000

[testing.geoip]
enabled = false
backend = "mmdb"
fallback = "none"
country_path = "mmdb/GeoLite2-Country.mmdb"
city_path = "mmdb/GeoLite2-City.mmdb"
asn_path = "mmdb/GeoLite2-ASN.mmdb"

[testing.geoip.remote]
provider = "ipwhois"
endpoint = ""
timeout_ms = 5000
api_key = ""
rate_limit_per_minute = 30

[testing.geoip.cache]
enabled = true
ttl_secs = 86400
max_entries = 10000
```

Real-delay status codes and inclusive ranges are combined with OR semantics.
Setting either acceptance field replaces the default `200-299` range. Valid
codes and range endpoints are `100-599`. When `follow_redirects` is enabled,
xrat follows at most 10 redirects and checks the terminal response; when it is
disabled, xrat checks the initial response so configured `3xx` statuses can
pass.

| Section           | Field                   | Type     | Default                                | Description                                                           |
| ----------------- | ----------------------- | -------- | -------------------------------------- | --------------------------------------------------------------------- |
| `[testing]`       | `concurrency`           | integer  | `0`                                    | Test workers (0 = auto)                                               |
| `[testing]`       | `order`                 | string[] | `["icmp", "real_delay", "download"]`   | Stage execution order; accepted: `icmp`, `tcp`, `real_delay`, `download` |
| `[testing]`       | `failure_policy`        | enum     | `continue`                             | Behavior on stage failure                                             |
| `[icmp]`          | `enabled`               | boolean  | `true`                                 | Enable ICMP stage                                                     |
| `[icmp]`          | `timeout`               | integer  | `3000`                                 | ICMP timeout (ms)                                                     |
| `[icmp]`          | `attempts`              | integer  | `3`                                    | ICMP attempt count                                                    |
| `[tcp]`           | `enabled`               | boolean  | `true`                                 | Enable TCP stage                                                      |
| `[tcp]`           | `timeout`               | integer  | `5000`                                 | TCP timeout (ms)                                                      |
| `[real_delay]`    | `enabled`               | boolean  | `true`                                 | Enable real-delay stage                                               |
| `[real_delay]`    | `url`                   | string   | `https://www.gstatic.com/generate_204` | Test URL                                                              |
| `[real_delay]`    | `timeout`               | integer  | `10000`                                | HTTP request timeout (ms)                                             |
| `[real_delay]`    | `accepted_status_codes` | integer[] | -                                     | Exact accepted HTTP status codes                                      |
| `[real_delay]`    | `accepted_status_ranges` | string[] | - (effective `200-299`)               | Inclusive accepted ranges in `START-END` form                         |
| `[real_delay]`    | `follow_redirects`      | boolean  | `true`                                 | Follow up to 10 redirects before checking status                      |
| `[download]`      | `enabled`               | boolean  | `false`                                | Enable download stage                                                 |
| `[download]`      | `url`                   | string   | -                                      | Download URL                                                          |
| `[download]`      | `timeout`               | integer  | `30000`                                | Download timeout (ms)                                                 |
| `[testing.geoip]` | `enabled`               | boolean  | `false`                                | Enable GeoIP enrichment                                               |
| `[testing.geoip]` | `backend`               | enum     | `mmdb`                                 | Lookup backend: `mmdb`, `ipwhois`, `ip-api`, `chain`                  |
| `[testing.geoip]` | `fallback`              | enum     | `none`                                 | Fallback backend when primary is `chain`: `ipwhois`, `ip-api`, `none` |
| `[testing.geoip]` | `country_path`          | string   | `mmdb/GeoLite2-Country.mmdb`           | Country MMDB path (relative to config)                                |
| `[testing.geoip]` | `city_path`             | string   | `mmdb/GeoLite2-City.mmdb`              | City MMDB path (relative to config)                                   |
| `[testing.geoip]` | `asn_path`              | string   | `mmdb/GeoLite2-ASN.mmdb`               | ASN MMDB path (relative to config)                                    |
| `[remote]`        | `provider`              | enum     | `ipwhois`                              | Remote provider: `ipwhois`, `ip-api`                                  |
| `[remote]`        | `endpoint`              | string   | `""` (uses provider default)           | Remote API endpoint override                                          |
| `[remote]`        | `timeout_ms`            | integer  | `5000`                                 | Remote request timeout in milliseconds                                |
| `[remote]`        | `api_key`               | string   | `""`                                   | API key (provider-specific)                                           |
| `[remote]`        | `rate_limit_per_minute` | integer  | `30`                                   | Max remote requests per minute                                        |
| `[cache]`         | `enabled`               | boolean  | `true`                                 | Enable in-memory caching                                              |
| `[cache]`         | `ttl_secs`              | integer  | `86400`                                | Cache entry TTL in seconds                                            |
| `[cache]`         | `max_entries`           | integer  | `10000`                                | Maximum cache entries                                                 |

Upload tests are enabled per invocation with `xrat test --upload-url <url>`.
There is no `[testing.upload]` config section; `--upload-timeout` overrides the
default 30-second upload timeout.

---

## Environment Variable References

Sensitive fields accept environment variable references:

```toml
# Literal value
password = "my-secret-password"

# Environment variable
password = { env = "XRAT_SOCKS_PASSWORD" }
```

Supported on these fields:

| Section                 | Field           |
| ----------------------- | --------------- |
| `[server]`              | `key`           |
| `[runtime.socks]`       | `auth.password` |
| `[runtime.shadowsocks]` | `password`      |
| `[database.postgres]`   | `user`          |
| `[database.postgres]`   | `password`      |

## Example Config

See `testdata/config.example.toml` in the repository for a complete example with
all sections and comments.
