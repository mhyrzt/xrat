# Configuration

xrat reads a TOML configuration file from the config directory. The default
location is `~/.config/xrat/config.toml`.

Override with `--config <path>` or the `XRAT_PATH` environment variable.

## Sections

| Section | Purpose |
|---------|---------|
| `[paths]` | Binary paths for xray, v2ray, sing-box |
| `[database]` | Backend selection and connection settings |
| `[runtime]` | Engine, rotation, logging, inbound settings |
| `[routing]` | Domain strategy, direct/block rules |
| `[geo]` | GeoIP auto-update settings |
| `[dns]` | DNS query strategy, servers, hosts |
| `[parser]` | Xray JSON schema validation mode |
| `[testing]` | Concurrency, stage order, per-stage settings |
| `[server]` | HTTP API host, port, API key |

## Example

```toml
[paths]
database = "db.sqlite"
# xray = "/usr/local/bin/xray"
# v2ray = "/usr/local/bin/v2ray"
# sing_box = "/usr/local/bin/sing-box"

[database]
backend = "sqlite"

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

[server]
enabled = false
host = "127.0.0.1"
port = 8080
key = { env = "XRAT_API_KEY" }

[runtime]
engine = "xray"
replace_active_session = true

[runtime.rotation]
enabled = false
interval_secs = 1800
health_trigger_enabled = true
cooldown_secs = 300
test_concurrency = 0
test_stages = ["real_delay", "download"]

[runtime.log]
enabled = true
mask = "none"
dir = "logs"
dns_log = false
level = "warning"
keep = true

[runtime.socks]
enabled = true
host = "0.0.0.0"
port = 1080
udp = true
auth = { enabled = true, username = "xrat", password = { env = "XRAT_SOCKS_PASSWORD" } }

[runtime.http]
enabled = false
host = "0.0.0.0"
port = 8080

[runtime.shadowsocks]
enabled = false
host = "0.0.0.0"
port = 1081
method = "aes-128-gcm"
password = { env = "XRAT_SHADOWSOCKS_PASSWORD" }
network = "tcp,udp"

[runtime.sniffing]
enabled = true
dest_override = ["http", "tls", "quic"]
route_only = true
metadata_only = false
domains_excluded = []
ips_excluded = []

[routing]
domain_strategy = "IPIfNonMatch"

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

[geo]
auto_update = false
update_interval_hours = 168

[[geo.profiles]]
name = "default"
geosite = "https://example.com/geosite.dat"
geoip = "https://example.com/geoip.dat"

[parser]
parse_mode = "strict"

[dns]
query_strategy = "UseSystem"
servers = ["8.8.8.8", "https://1.1.1.1/dns-query"]
use_system_hosts = true
disable_cache = false
disable_fallback = false
enable_parallel_query = true

[dns.hosts]
"domain:example.test" = "127.0.0.1"

[testing]
concurrency = 0
order = ["icmp", "real_delay", "download"]
failure_policy = "continue"

[testing.real_delay]
enabled = true
url = "https://www.google.com/generate_204"
timeout = 10_000

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
```

## Secret Values

Sensitive fields accept either a literal string or an environment variable
reference:

```toml
password = "literal-value"
password = { env = "XRAT_SOCKS_PASSWORD" }
```

## Full Reference

See [config-file](../05-reference/config-file.md) for the complete field
reference with all defaults and accepted values.
