#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SettingHelp {
    pub(crate) description: &'static str,
    pub(crate) value_hint: &'static str,
    pub(crate) example: &'static str,
}

const fn help(
    description: &'static str,
    value_hint: &'static str,
    example: &'static str,
) -> SettingHelp {
    SettingHelp {
        description,
        value_hint,
        example,
    }
}

pub(super) const FALLBACK: SettingHelp = help(
    "Configures this xrat setting.",
    "Enter a value accepted by config.toml.",
    "value = \"example\"",
);

pub(super) fn for_path(path: &str) -> Option<SettingHelp> {
    Some(match path {
        "runtime.engine" => help(
            "Selects the proxy engine used to generate and run configurations.",
            "",
            "engine = \"xray\"",
        ),
        "runtime.replace_active_session" => help(
            "Allows a new connection to replace the currently active managed session.",
            "",
            "replace_active_session = true",
        ),
        "runtime.rotation.enabled" => help(
            "Enables automatic selection and replacement of the active proxy.",
            "",
            "enabled = true",
        ),
        "runtime.rotation.interval_secs" => help(
            "Sets how often scheduled rotation evaluates a replacement candidate.",
            "Positive whole seconds.",
            "interval_secs = 1800",
        ),
        "runtime.rotation.health_trigger_enabled" => help(
            "Allows failed health checks to trigger an early rotation.",
            "",
            "health_trigger_enabled = true",
        ),
        "runtime.rotation.cooldown_secs" => help(
            "Prevents repeated health-triggered switches during this cooldown window.",
            "Zero or more whole seconds.",
            "cooldown_secs = 300",
        ),
        "runtime.rotation.test_concurrency" => help(
            "Limits parallel candidate tests performed before rotation.",
            "0 selects an automatic worker count; positive values set an exact count.",
            "test_concurrency = 0",
        ),
        "runtime.rotation.test_stages" => help(
            "Chooses the test stages used to rank rotation candidates, in execution order.",
            "Comma-separated: icmp, tcp, real_delay, download.",
            "test_stages = [\"icmp\", \"real_delay\"]",
        ),
        "runtime.rotation.refresh_subscriptions" => help(
            "Refreshes URL subscriptions before choosing a rotation candidate.",
            "",
            "refresh_subscriptions = false",
        ),
        "runtime.log.enabled" => help(
            "Enables proxy-engine log files for managed runtime sessions.",
            "",
            "enabled = true",
        ),
        "runtime.log.mask" => help(
            "Controls how much of IP addresses is hidden in engine logs.",
            "quarter, half, full, or none.",
            "mask = \"none\"",
        ),
        "runtime.log.dir" => help(
            "Sets the directory where managed proxy logs are written.",
            "Absolute path or path relative to the xrat runtime directory.",
            "dir = \"logs\"",
        ),
        "runtime.log.dns_log" => help(
            "Includes proxy-engine DNS query activity in its logs.",
            "",
            "dns_log = false",
        ),
        "runtime.log.level" => help(
            "Sets the minimum severity written by the proxy engine.",
            "",
            "level = \"warning\"",
        ),
        "runtime.log.keep" => help(
            "Keeps generated engine log files after the runtime session stops.",
            "",
            "keep = true",
        ),
        "runtime.socks.enabled" => help(
            "Enables the local SOCKS5 inbound for applications using the proxy.",
            "",
            "enabled = true",
        ),
        "runtime.socks.host" => help(
            "Sets the address on which the SOCKS5 inbound listens.",
            "IP address or hostname; use 127.0.0.1 for local-only access.",
            "host = \"127.0.0.1\"",
        ),
        "runtime.socks.port" => help(
            "Sets the local listening port for the SOCKS5 inbound.",
            "Whole number from 1 to 65535; must not conflict with another enabled inbound.",
            "port = 18200",
        ),
        "runtime.socks.udp" => help(
            "Allows UDP traffic through the SOCKS5 inbound.",
            "",
            "udp = true",
        ),
        "runtime.socks.auth.enabled" => help(
            "Requires username and password authentication on the SOCKS5 inbound.",
            "",
            "enabled = true",
        ),
        "runtime.socks.auth.username" => help(
            "Sets the SOCKS5 authentication username when authentication is enabled.",
            "Literal text.",
            "username = \"xrat\"",
        ),
        "runtime.socks.auth.password" => help(
            "Sets the SOCKS5 authentication password when authentication is enabled.",
            "Literal text or env:VARIABLE_NAME; an environment variable is safer.",
            "password = { env = \"XRAT_SOCKS_PASSWORD\" }",
        ),
        "runtime.http.enabled" => help(
            "Enables the local HTTP proxy inbound.",
            "",
            "enabled = false",
        ),
        "runtime.http.host" => help(
            "Sets the address on which the HTTP proxy listens.",
            "IP address or hostname; use 127.0.0.1 for local-only access.",
            "host = \"127.0.0.1\"",
        ),
        "runtime.http.port" => help(
            "Sets the local listening port for the HTTP proxy.",
            "Whole number from 1 to 65535; must not conflict with another enabled inbound.",
            "port = 18201",
        ),
        "runtime.shadowsocks.enabled" => help(
            "Enables the local Shadowsocks inbound.",
            "",
            "enabled = false",
        ),
        "runtime.shadowsocks.host" => help(
            "Sets the address on which the Shadowsocks inbound listens.",
            "IP address or hostname; use 127.0.0.1 for local-only access.",
            "host = \"127.0.0.1\"",
        ),
        "runtime.shadowsocks.port" => help(
            "Sets the local listening port for the Shadowsocks inbound.",
            "Whole number from 1 to 65535; must not conflict with another enabled inbound.",
            "port = 18202",
        ),
        "runtime.shadowsocks.method" => help(
            "Selects the cipher used by clients connecting to the Shadowsocks inbound.",
            "",
            "method = \"aes-128-gcm\"",
        ),
        "runtime.shadowsocks.password" => help(
            "Sets the password required by the Shadowsocks inbound.",
            "Literal text or env:VARIABLE_NAME; an environment variable is safer.",
            "password = { env = \"XRAT_SHADOWSOCKS_PASSWORD\" }",
        ),
        "runtime.shadowsocks.network" => help(
            "Chooses which transport families the Shadowsocks inbound accepts.",
            "",
            "network = \"tcp,udp\"",
        ),
        "runtime.sniffing.enabled" => help(
            "Enables destination-protocol sniffing for inbound traffic.",
            "",
            "enabled = true",
        ),
        "runtime.sniffing.dest_override" => help(
            "Lists protocols whose detected destination may replace the original destination.",
            "Comma-separated protocol names such as http, tls, and quic.",
            "dest_override = [\"http\", \"tls\", \"quic\"]",
        ),
        "runtime.sniffing.route_only" => help(
            "Uses sniffed destinations for routing decisions without rewriting the connection destination.",
            "",
            "route_only = true",
        ),
        "runtime.sniffing.metadata_only" => help(
            "Restricts sniffing to connection metadata instead of inspecting payload data.",
            "",
            "metadata_only = false",
        ),
        "runtime.sniffing.domains_excluded" => help(
            "Excludes matching domains from destination sniffing.",
            "Comma-separated domain patterns; an empty list disables exclusions.",
            "domains_excluded = [\"example.com\"]",
        ),
        "runtime.sniffing.ips_excluded" => help(
            "Excludes matching IP addresses or networks from destination sniffing.",
            "Comma-separated IP addresses or CIDRs; an empty list disables exclusions.",
            "ips_excluded = [\"192.168.0.0/16\"]",
        ),
        "runtime.stats.enabled" => help(
            "Enables the local engine statistics endpoint sampled by the TUI.",
            "",
            "enabled = true",
        ),
        "runtime.stats.host" => help(
            "Sets the listening address for the local statistics controller.",
            "Use a loopback address unless remote access is intentionally required.",
            "host = \"127.0.0.1\"",
        ),
        "runtime.stats.port" => help(
            "Sets the listening port for the statistics controller.",
            "Whole number from 1 to 65535; must not conflict with another local listener.",
            "port = 10085",
        ),
        "runtime.mux.enabled" => help(
            "Enables multiplexing multiple logical connections over fewer proxy connections.",
            "",
            "enabled = false",
        ),
        "runtime.mux.concurrency" => help(
            "Sets logical connections per Mux session; high values may hurt bulk throughput.",
            "-1 disables TCP Mux, 0 uses the engine default, and 1-128 sets an exact limit.",
            "concurrency = 8",
        ),
        "runtime.mux.xudp_concurrency" => help(
            "Sets XUDP aggregation concurrency for UDP traffic under Mux.",
            "-1 opts UDP out of Mux, 0 uses the legacy path, and 1-1024 sets a limit.",
            "xudp_concurrency = 0",
        ),
        "runtime.mux.xudp_proxy_udp443" => help(
            "Controls how XUDP handles QUIC traffic on UDP port 443.",
            "",
            "xudp_proxy_udp443 = \"reject\"",
        ),
        "runtime.fragment.enabled" => help(
            "Enables outbound TCP fragmentation, which may help on some filtered networks.",
            "",
            "enabled = false",
        ),
        "runtime.fragment.packets_mode" => help(
            "Chooses whether to fragment the TLS ClientHello automatically or use a packet range.",
            "",
            "packets_mode = \"tlshello\"",
        ),
        "runtime.fragment.packets" => help(
            "Sets the inclusive packet-index range used when packet mode is range.",
            "Two positive whole numbers: minimum, maximum.",
            "packets = [1, 3]",
        ),
        "runtime.fragment.length" => help(
            "Sets the minimum and maximum fragment length in bytes.",
            "Two positive whole numbers: minimum, maximum.",
            "length = [100, 200]",
        ),
        "runtime.fragment.interval" => help(
            "Sets the minimum and maximum delay between fragments in milliseconds.",
            "Two non-negative whole numbers: minimum, maximum.",
            "interval = [10, 20]",
        ),
        "runtime.network.interface" => help(
            "Binds outbound sockets to this operating-system network interface.",
            "Interface name, or an empty string for automatic routing.",
            "interface = \"eth0\"",
        ),
        "runtime.network.bind_address" => help(
            "Binds outbound sockets to this local source address.",
            "IP address, or an empty string for automatic selection.",
            "bind_address = \"192.168.1.10\"",
        ),
        "runtime.network.mark" => help(
            "Applies a Linux socket mark used by policy routing and firewall rules.",
            "Non-negative whole number; 0 disables marking.",
            "mark = 255",
        ),
        "runtime.network.listen_interface" => help(
            "Binds generated inbound listeners to this network interface where supported.",
            "Interface name, or an empty string for no interface binding.",
            "listen_interface = \"eth0\"",
        ),
        "subscriptions.auto_refresh" => help(
            "Enables periodic refresh of URL-backed subscriptions in the daemon.",
            "",
            "auto_refresh = true",
        ),
        "subscriptions.refresh_interval_hours" => help(
            "Sets the interval between automatic subscription refresh attempts.",
            "Whole hours; values below 1 are treated as 1 hour.",
            "refresh_interval_hours = 24",
        ),
        "routing.domain_strategy" => help(
            "Controls when routing rules resolve domain names to IP addresses.",
            "",
            "domain_strategy = \"IPIfNonMatch\"",
        ),
        "routing.direct.domain" => help(
            "Routes matching domain rules directly instead of through the proxy.",
            "Comma-separated Xray domain rules; an empty list disables this match type.",
            "domain = [\"domain:example.com\"]",
        ),
        "routing.direct.ip" => help(
            "Routes matching IP addresses or networks directly.",
            "Comma-separated IP addresses or CIDRs.",
            "ip = [\"192.168.0.0/16\"]",
        ),
        "routing.direct.geosite" => help(
            "Routes domains from matching geosite categories directly.",
            "Comma-separated geosite category names.",
            "geosite = [\"private\"]",
        ),
        "routing.direct.geoip" => help(
            "Routes addresses from matching GeoIP categories directly.",
            "Comma-separated GeoIP category names.",
            "geoip = [\"private\"]",
        ),
        "routing.block.domain" => help(
            "Blocks connections matching these domain rules.",
            "Comma-separated Xray domain rules.",
            "domain = [\"domain:ads.example\"]",
        ),
        "routing.block.ip" => help(
            "Blocks connections matching these IP addresses or networks.",
            "Comma-separated IP addresses or CIDRs.",
            "ip = [\"203.0.113.0/24\"]",
        ),
        "routing.block.geosite" => help(
            "Blocks domains from matching geosite categories.",
            "Comma-separated geosite category names.",
            "geosite = [\"category-ads-all\"]",
        ),
        "routing.block.geoip" => help(
            "Blocks addresses from matching GeoIP categories.",
            "Comma-separated GeoIP category names.",
            "geoip = [\"cn\"]",
        ),
        "dns.query_strategy" => help(
            "Controls which address families the generated DNS resolver requests.",
            "",
            "query_strategy = \"UseSystem\"",
        ),
        "dns.servers" => help(
            "Sets upstream DNS resolvers used by generated proxy configurations.",
            "Comma-separated IP, hostname, HTTPS, or other engine-supported DNS endpoints.",
            "servers = [\"1.1.1.1\", \"https://dns.google/dns-query\"]",
        ),
        "dns.use_system_hosts" => help(
            "Allows generated DNS configuration to consult the operating-system hosts file.",
            "",
            "use_system_hosts = true",
        ),
        "dns.disable_cache" => help(
            "Disables the proxy engine's DNS response cache.",
            "",
            "disable_cache = false",
        ),
        "dns.disable_fallback" => help(
            "Disables fallback DNS resolution when the primary resolver path cannot answer.",
            "",
            "disable_fallback = false",
        ),
        "dns.enable_parallel_query" => help(
            "Allows parallel queries to eligible DNS resolvers for faster responses.",
            "",
            "enable_parallel_query = true",
        ),
        "testing.concurrency" => help(
            "Limits how many configurations the test pipeline evaluates in parallel.",
            "0 selects an automatic worker count; positive values set an exact count.",
            "concurrency = 0",
        ),
        "testing.order" => help(
            "Sets the connection-test stages and their execution order.",
            "Comma-separated: icmp, tcp, real_delay, download; each stage may appear once.",
            "order = [\"icmp\", \"real_delay\", \"download\"]",
        ),
        "testing.failure_policy" => help(
            "Controls whether later stages run after an earlier test fails.",
            "",
            "failure_policy = \"continue\"",
        ),
        "testing.real_delay.enabled" => help(
            "Enables an HTTP request through the proxy to measure real end-to-end delay.",
            "",
            "enabled = true",
        ),
        "testing.real_delay.url" => help(
            "Sets the HTTP endpoint used for real-delay measurements.",
            "HTTP or HTTPS URL.",
            "url = \"https://www.gstatic.com/generate_204\"",
        ),
        "testing.real_delay.timeout" => help(
            "Sets the maximum duration of a real-delay request.",
            "Positive whole milliseconds.",
            "timeout = 10000",
        ),
        "testing.real_delay.accepted_status_codes" => help(
            "Accepts these exact HTTP status codes as successful real-delay responses.",
            "Comma-separated whole numbers from 100 to 599; none uses range/default behavior.",
            "accepted_status_codes = [200, 204]",
        ),
        "testing.real_delay.accepted_status_ranges" => help(
            "Accepts HTTP response codes inside these inclusive ranges.",
            "Comma-separated START-END ranges within 100-599.",
            "accepted_status_ranges = [\"200-299\"]",
        ),
        "testing.real_delay.follow_redirects" => help(
            "Follows HTTP redirects before checking the final response status.",
            "",
            "follow_redirects = true",
        ),
        "testing.icmp.enabled" => help(
            "Enables ICMP latency and reachability checks.",
            "",
            "enabled = true",
        ),
        "testing.icmp.attempts" => help(
            "Sets how many ICMP echo requests are sent per configuration.",
            "Positive whole number.",
            "attempts = 3",
        ),
        "testing.icmp.timeout" => help(
            "Sets the timeout for each ICMP test attempt.",
            "Positive whole milliseconds.",
            "timeout = 3000",
        ),
        "testing.download.enabled" => help(
            "Enables download-throughput measurement through the proxy.",
            "",
            "enabled = false",
        ),
        "testing.download.url" => help(
            "Sets the file URL used for download-throughput tests.",
            "HTTP or HTTPS URL serving a sufficiently large file.",
            "url = \"https://cachefly.cachefly.net/50mb.test\"",
        ),
        "testing.download.timeout" => help(
            "Sets the maximum duration of a download-throughput test.",
            "Positive whole milliseconds.",
            "timeout = 30000",
        ),
        "testing.tcp.enabled" => help(
            "Enables TCP connection latency and reachability checks.",
            "",
            "enabled = true",
        ),
        "testing.tcp.timeout" => help(
            "Sets the maximum duration of a TCP connection attempt.",
            "Positive whole milliseconds.",
            "timeout = 5000",
        ),
        "testing.geoip.enabled" => help(
            "Enables GeoIP enrichment of tested endpoints.",
            "",
            "enabled = false",
        ),
        "testing.geoip.backend" => help(
            "Selects the primary source used for GeoIP lookup.",
            "",
            "backend = \"mmdb\"",
        ),
        "testing.geoip.fallback" => help(
            "Selects the lookup source used when the primary GeoIP backend fails.",
            "",
            "fallback = \"none\"",
        ),
        "testing.geoip.country_path" => help(
            "Sets the MaxMind country database used by the MMDB backend.",
            "Absolute path or path resolved through the configured MMDB directory.",
            "country_path = \"mmdb/GeoLite2-Country.mmdb\"",
        ),
        "testing.geoip.city_path" => help(
            "Sets the MaxMind city database used by the MMDB backend.",
            "Absolute path or path resolved through the configured MMDB directory.",
            "city_path = \"mmdb/GeoLite2-City.mmdb\"",
        ),
        "testing.geoip.asn_path" => help(
            "Sets the MaxMind ASN database used by the MMDB backend.",
            "Absolute path or path resolved through the configured MMDB directory.",
            "asn_path = \"mmdb/GeoLite2-ASN.mmdb\"",
        ),
        "testing.geoip.remote.provider" => help(
            "Selects the remote service used for GeoIP lookup.",
            "",
            "provider = \"ip-whois\"",
        ),
        "testing.geoip.remote.endpoint" => help(
            "Overrides the selected remote GeoIP provider endpoint.",
            "HTTP or HTTPS URL; empty uses the provider default.",
            "endpoint = \"https://ipwho.is/{ip}\"",
        ),
        "testing.geoip.remote.timeout_ms" => help(
            "Sets the timeout for each remote GeoIP request.",
            "Positive whole milliseconds.",
            "timeout_ms = 5000",
        ),
        "testing.geoip.remote.api_key" => help(
            "Sets the credential supplied to a remote GeoIP provider when required.",
            "Literal provider credential; the current value remains masked in the TUI.",
            "api_key = \"provider-key\"",
        ),
        "testing.geoip.remote.rate_limit_per_minute" => help(
            "Limits remote GeoIP requests to protect provider quotas.",
            "Positive whole requests per minute.",
            "rate_limit_per_minute = 30",
        ),
        "testing.geoip.cache.enabled" => help(
            "Caches GeoIP lookup results to reduce repeated local or remote work.",
            "",
            "enabled = true",
        ),
        "testing.geoip.cache.ttl_secs" => help(
            "Sets how long cached GeoIP results remain valid.",
            "Positive whole seconds.",
            "ttl_secs = 86400",
        ),
        "testing.geoip.cache.max_entries" => help(
            "Caps the number of GeoIP results retained in the cache.",
            "Positive whole number of entries.",
            "max_entries = 10000",
        ),
        "server.enabled" => help("Enables the xrat HTTP API server.", "", "enabled = false"),
        "server.host" => help(
            "Sets the address on which the HTTP API server listens.",
            "IP address or hostname; use 127.0.0.1 for local-only access.",
            "host = \"127.0.0.1\"",
        ),
        "server.port" => help(
            "Sets the listening port for the HTTP API server.",
            "Whole number from 1 to 65535.",
            "port = 8080",
        ),
        "server.key" => help(
            "Sets the API authentication key required by protected endpoints.",
            "Literal text or env:VARIABLE_NAME; an environment variable is safer.",
            "key = { env = \"XRAT_API_KEY\" }",
        ),
        "server.pac_enabled" => help(
            "Enables serving the generated proxy auto-configuration file.",
            "",
            "pac_enabled = true",
        ),
        "server.pac_allowed_hosts" => help(
            "Limits Host headers accepted by the PAC endpoint.",
            "Comma-separated hostnames or IP addresses.",
            "pac_allowed_hosts = [\"localhost\", \"127.0.0.1\"]",
        ),
        "parser.parse_mode" => help(
            "Controls how strictly imported Xray JSON is checked for unknown fields.",
            "",
            "parse_mode = \"strict\"",
        ),
        _ => return None,
    })
}
