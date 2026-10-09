use super::prelude::*;

pub(crate) fn validate_config(config: &AppConfig, errors: &mut Vec<Diagnostic>) {
    validate_runtime(config, errors);
    validate_routing(&config.routing, errors);
    validate_database(config, errors);
    validate_testing(&config.testing, errors);
    validate_server(config, errors);
    if let Err(error) =
        crate::app::services::runtime_tuning::dns_validation::validate_listener_ports(
            &config.dns,
            &config.runtime,
        )
    {
        errors.push(Diagnostic::new(
            "[dns.listener].port",
            error,
            "Managed listeners need separate ports.",
            "choose an unused DNS listener port.",
        ));
    }
    if let Err(error) = crate::app::services::runtime_tuning::dns_validation::validate_tun_ranges(
        &config.dns,
        &config.runtime.tun,
    ) {
        errors.push(Diagnostic::new(
            "[dns.fakeip]",
            error,
            "FakeIP ranges must be captured without interface conflicts.",
            "choose a separate reserved FakeIP pool.",
        ));
    }
    if let Err(error) = crate::app::services::runtime_tuning::dns_validation::validate(
        &config.dns,
        &config.runtime.engine,
        config.runtime.tun.enabled,
    ) {
        errors.push(Diagnostic::new(
            "[dns]",
            error,
            "DNS policy must match engine capabilities.",
            "correct the named DNS settings before connecting.",
        ));
    }
}

pub(crate) fn validate_routing(routing: &RoutingSettings, errors: &mut Vec<Diagnostic>) {
    if !matches!(
        routing.domain_strategy.as_str(),
        "AsIs" | "IPIfNonMatch" | "IPOnDemand"
    ) {
        errors.push(Diagnostic::new(
            "[routing].domain_strategy",
            format!("unsupported strategy: {}", routing.domain_strategy),
            "Xray and V2Ray accept only their documented routing domain strategies.",
            "use one of: AsIs, IPIfNonMatch, IPOnDemand.",
        ));
    }

    validate_route_list("[routing.direct]", &routing.direct, errors);
    validate_route_list("[routing.block]", &routing.block, errors);
}

pub(crate) fn validate_route_list(prefix: &str, routes: &RouteList, errors: &mut Vec<Diagnostic>) {
    for (name, values) in [
        ("domain", &routes.domain),
        ("ip", &routes.ip),
        ("geosite", &routes.geosite),
        ("geoip", &routes.geoip),
    ] {
        if values.iter().any(|value| value.trim().is_empty()) {
            errors.push(Diagnostic::new(
                format!("{prefix}.{name}"),
                "contains an empty routing rule",
                "empty rules are invalid and can make the generated engine configuration fail native validation.",
                "remove blank entries or replace them with a valid routing rule.",
            ));
        }
    }
}

pub(crate) fn validate_app_config(config: &AppConfig) -> Vec<String> {
    let mut diagnostics = Vec::new();
    validate_config(config, &mut diagnostics);
    diagnostics
        .into_iter()
        .map(|diagnostic| {
            if diagnostic.field.is_empty() {
                format!("{}; {}", diagnostic.problem, diagnostic.fix)
            } else {
                format!(
                    "{}: {}; {}",
                    diagnostic.field, diagnostic.problem, diagnostic.fix
                )
            }
        })
        .collect()
}

pub(crate) fn validate_runtime(config: &AppConfig, errors: &mut Vec<Diagnostic>) {
    let runtime = &config.runtime;
    if !matches!(runtime.engine.as_str(), "xray" | "v2ray" | "sing-box") {
        errors.push(Diagnostic::new(
            "[runtime].engine",
            format!("unsupported engine: {}", runtime.engine),
            "the engine selects which proxy core generates and runs the runtime config.",
            "use one of: xray, v2ray, sing-box.",
        ));
    }

    if runtime.rotation.test_concurrency < 0 {
        errors.push(Diagnostic::new(
            "[runtime.rotation].test_concurrency",
            format!("value is {}", runtime.rotation.test_concurrency),
            "concurrency cannot be negative; 0 means an automatic default.",
            "use 0 for the default, or a positive number of parallel tests.",
        ));
    }
    if runtime.rotation.health_failure_threshold == 0 {
        errors.push(Diagnostic::new(
            "[runtime.rotation].health_failure_threshold",
            "value is 0",
            "data-plane health recovery requires at least one failed probe.",
            "use 3 for the recommended failure threshold, or another positive number.",
        ));
    }

    for stage in &runtime.rotation.test_stages {
        if ConnectionTestStage::from_config_str(stage).is_none() {
            errors.push(Diagnostic::new(
                "[runtime.rotation].test_stages",
                format!("unsupported stage: {stage}"),
                "rotation can only run stages the tester knows about.",
                "use accepted stages: icmp (alias ping), real_delay (alias real-delay), download (alias download-speed), tcp.",
            ));
        }
    }

    let mut enabled_ports = BTreeSet::new();
    validate_inbound(
        "[runtime.socks]",
        runtime.socks.enabled,
        &runtime.socks.host,
        runtime.socks.port,
        &mut enabled_ports,
        errors,
    );
    validate_inbound(
        "[runtime.http]",
        runtime.http.enabled,
        &runtime.http.host,
        runtime.http.port,
        &mut enabled_ports,
        errors,
    );
    validate_inbound(
        "[runtime.shadowsocks]",
        runtime.shadowsocks.enabled,
        &runtime.shadowsocks.host,
        runtime.shadowsocks.port,
        &mut enabled_ports,
        errors,
    );

    if runtime.socks.auth.enabled {
        if runtime
            .socks
            .auth
            .username
            .as_deref()
            .unwrap_or_default()
            .is_empty()
        {
            errors.push(Diagnostic::new(
                "[runtime.socks.auth].username",
                "username is empty",
                "SOCKS auth requires a username when enabled.",
                "set a non-empty username, or set [runtime.socks.auth].enabled = false.",
            ));
        }
        validate_secret(
            "[runtime.socks.auth].password",
            runtime.socks.auth.password.as_ref(),
            errors,
        );
    }

    if runtime.shadowsocks.enabled {
        if runtime.shadowsocks.method.is_empty() {
            errors.push(Diagnostic::new(
                "[runtime.shadowsocks].method",
                "method is empty",
                "the Shadowsocks inbound needs a cipher method to encrypt traffic.",
                "set a supported method such as aes-256-gcm or chacha20-ietf-poly1305.",
            ));
        }
        validate_secret(
            "[runtime.shadowsocks].password",
            Some(&runtime.shadowsocks.password),
            errors,
        );
        for network in runtime.shadowsocks.network.split(',') {
            if !matches!(network.trim(), "tcp" | "udp") {
                errors.push(Diagnostic::new(
                    "[runtime.shadowsocks].network",
                    format!("unsupported value: {network}"),
                    "network selects which transports the inbound accepts.",
                    "use tcp, udp, or a comma-separated combination like \"tcp,udp\".",
                ));
            }
        }
    }

    validate_mux(&runtime.mux, errors);
    validate_fragment(&runtime.fragment, errors);
    validate_network(&runtime.network, errors);
    validate_tun(&runtime.tun, &runtime.engine, errors);
}

pub(crate) fn validate_mux(mux: &crate::app::config::MuxSettings, errors: &mut Vec<Diagnostic>) {
    if !(-1..=128).contains(&mux.concurrency) {
        errors.push(Diagnostic::new(
            "[runtime.mux].concurrency",
            format!("value is {}", mux.concurrency),
            "Xray accepts 1..=128 connections per Mux session; 0 means the default of 8 and -1 disables TCP Mux.",
            "use -1, 0, or a value in 1..=128.",
        ));
    }
    if !(-1..=1024).contains(&mux.xudp_concurrency) {
        errors.push(Diagnostic::new(
            "[runtime.mux].xudp_concurrency",
            format!("value is {}", mux.xudp_concurrency),
            "Xray accepts 1..=1024 for XUDP aggregation; 0 keeps the legacy path and -1 opts UDP out of Mux.",
            "use -1, 0, or a value in 1..=1024.",
        ));
    }
    if !matches!(mux.xudp_proxy_udp443.as_str(), "reject" | "allow" | "skip") {
        errors.push(Diagnostic::new(
            "[runtime.mux].xudp_proxy_udp443",
            format!("unsupported value: {}", mux.xudp_proxy_udp443),
            "this controls how QUIC/UDP 443 traffic is handled when XUDP is active.",
            "use one of: reject, allow, skip.",
        ));
    }
}

pub(crate) fn validate_fragment(
    fragment: &crate::app::config::FragmentSettings,
    errors: &mut Vec<Diagnostic>,
) {
    match fragment.packets_mode.trim() {
        "tlshello" => {}
        "range" => {
            if fragment.packets[0] == 0 || fragment.packets[0] > fragment.packets[1] {
                errors.push(Diagnostic::new(
                    "[runtime.fragment].packets",
                    format!(
                        "value is [{}, {}]",
                        fragment.packets[0], fragment.packets[1]
                    ),
                    "in range mode, packets is a positive write range with min <= max.",
                    "use a range like [1, 3] where both values are >= 1.",
                ));
            }
        }
        other => {
            errors.push(Diagnostic::new(
                "[runtime.fragment].packets_mode",
                format!("unsupported value: {other}"),
                "packets_mode selects how outgoing writes are fragmented.",
                "use \"tlshello\" or \"range\".",
            ));
        }
    }
    if fragment.length[0] == 0 || fragment.length[0] > fragment.length[1] {
        errors.push(Diagnostic::new(
            "[runtime.fragment].length",
            format!("value is [{}, {}]", fragment.length[0], fragment.length[1]),
            "length is a positive byte range with min <= max.",
            "use a range like [100, 200] where both values are >= 1.",
        ));
    }
    if fragment.interval[0] > fragment.interval[1] {
        errors.push(Diagnostic::new(
            "[runtime.fragment].interval",
            format!(
                "value is [{}, {}]",
                fragment.interval[0], fragment.interval[1]
            ),
            "interval is a millisecond range with min <= max.",
            "use a range like [10, 20].",
        ));
    }
}

pub(crate) fn validate_network(
    network: &crate::app::config::NetworkSettings,
    errors: &mut Vec<Diagnostic>,
) {
    if !network.bind_address.trim().is_empty()
        && network
            .bind_address
            .trim()
            .parse::<std::net::IpAddr>()
            .is_err()
    {
        errors.push(Diagnostic::new(
            "[runtime.network].bind_address",
            format!("invalid address: {}", network.bind_address),
            "bind_address must be a literal source IP. Note: the Xray engine cannot honor source-IP binding and will ignore it.",
            "leave it empty, or set a valid IPv4/IPv6 address.",
        ));
    }
    if network.mark < 0 {
        errors.push(Diagnostic::new(
            "[runtime.network].mark",
            format!("value is {}", network.mark),
            "mark is an fwmark applied to outbound sockets and cannot be negative; 0 means unset.",
            "use 0 to disable, or a positive fwmark value.",
        ));
    }
}

pub(crate) fn validate_tun(
    tun: &crate::app::config::TunSettings,
    engine: &str,
    errors: &mut Vec<Diagnostic>,
) {
    validate_tun_app_list("[runtime.tun].blacklist", &tun.blacklist, errors);
    validate_tun_app_list("[runtime.tun].whitelist", &tun.whitelist, errors);

    if !tun.enabled {
        return;
    }
    if !matches!(engine, "xray" | "sing-box") {
        errors.push(Diagnostic::new(
            "[runtime.tun].enabled",
            format!("TUN capture is not supported by the {engine} engine"),
            "only the xray and sing-box engines emit a tun inbound.",
            "use [runtime].engine = \"xray\" or \"sing-box\", or disable [runtime.tun].",
        ));
    }
    if tun.interface_name.trim().is_empty() {
        errors.push(Diagnostic::new(
            "[runtime.tun].interface_name",
            "interface_name is empty",
            "the TUN interface needs a name to create and clean up.",
            "set a name such as xrat0.",
        ));
    }
    if tun.interface_name.len() > 15
        || tun
            .interface_name
            .chars()
            .any(|value| value.is_whitespace() || matches!(value, '/' | '\0'))
    {
        errors.push(Diagnostic::new(
            "[runtime.tun].interface_name",
            "invalid Linux interface name",
            "interface names require 1..15 bytes without whitespace, slash or NUL.",
            "choose a name such as xrat0.",
        ));
    }
    if engine == "xray" && (tun.stack != "system" || tun.strict_route) {
        errors.push(Diagnostic::new(
            "[runtime.tun]",
            "Xray cannot apply stack selection or strict_route",
            "these options are sing-box-only; Xray uses its fixed native stack.",
            "leave stack at system and strict_route false, or use sing-box.",
        ));
    }
    if !(1280..=65535).contains(&tun.mtu) {
        errors.push(Diagnostic::new(
            "[runtime.tun].mtu",
            format!("value is {}", tun.mtu),
            "the TUN MTU must be a whole number in 1280..=65535.",
            "use 1500 for typical Ethernet links.",
        ));
    }
    if !matches!(tun.stack.as_str(), "system" | "gvisor" | "mixed") {
        errors.push(Diagnostic::new(
            "[runtime.tun].stack",
            format!("unsupported stack: {}", tun.stack),
            "the sing-box TUN stack selects how captured packets are processed.",
            "use one of: system, gvisor, mixed.",
        ));
    }
    if tun.address.is_empty() {
        errors.push(Diagnostic::new(
            "[runtime.tun].address",
            "no address configured",
            "the TUN interface needs at least one address CIDR.",
            "set an address such as 172.19.0.1/30.",
        ));
    }
    for address in &tun.address {
        if !is_valid_cidr(address) {
            errors.push(Diagnostic::new(
                "[runtime.tun].address",
                format!("invalid CIDR: {address}"),
                "TUN addresses must be IPv4/IPv6 CIDR prefixes.",
                "use a value like 172.19.0.1/30.",
            ));
        }
    }
    for address in &tun.route_exclude_address {
        if !is_valid_cidr(address) {
            errors.push(Diagnostic::new(
                "[runtime.tun].route_exclude_address",
                format!("invalid CIDR: {address}"),
                "route exclusions must be IPv4/IPv6 CIDR prefixes.",
                "use a value like 192.168.0.0/16.",
            ));
        }
    }
    if engine != "sing-box" && !tun.route_exclude_address.is_empty() {
        errors.push(Diagnostic::new(
            "[runtime.tun].route_exclude_address",
            "not supported by the xray engine",
            "the Xray TUN inbound has no route-exclusion option; only sing-box can exclude destinations from capture.",
            "remove the exclusions, or switch [runtime].engine to \"sing-box\".",
        ));
    }
}

fn validate_tun_app_list(field: &str, entries: &[String], errors: &mut Vec<Diagnostic>) {
    for entry in entries {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            errors.push(Diagnostic::new(
                field,
                "contains an empty application entry",
                "empty split-tunneling application rules are invalid.",
                "remove blank entries or specify a process name, absolute path (/usr/bin/app), directory (/opt/app/), or .desktop ID.",
            ));
            continue;
        }
        if trimmed.contains('\0') || trimmed.contains('\n') || trimmed.contains('\r') {
            errors.push(Diagnostic::new(
                field,
                format!("entry {entry:?} contains control characters"),
                "process names and paths cannot contain newline or NUL characters.",
                "use a single-line process name, path, or .desktop ID.",
            ));
            continue;
        }
        if trimmed.contains('/') && !trimmed.starts_with('/') && !trimmed.starts_with("desktop:") {
            errors.push(Diagnostic::new(
                field,
                format!("relative path entry is not allowed: {trimmed}"),
                "path-based split-tunneling rules must be absolute paths starting with '/'.",
                "use an absolute executable path like /usr/bin/curl, a directory prefix like /opt/app/, or a bare process name.",
            ));
        }
    }
}

fn is_valid_cidr(value: &str) -> bool {
    let Some((address, prefix)) = value.trim().split_once('/') else {
        return false;
    };
    let Ok(address) = address.parse::<std::net::IpAddr>() else {
        return false;
    };
    let Ok(prefix) = prefix.parse::<u8>() else {
        return false;
    };
    let max = if address.is_ipv4() { 32 } else { 128 };
    prefix <= max
}

pub(crate) fn validate_database(config: &AppConfig, errors: &mut Vec<Diagnostic>) {
    if config.database.backend != DatabaseBackend::Postgres {
        return;
    }

    validate_secret(
        "[database.postgres].user",
        Some(&config.database.postgres.user),
        errors,
    );
    if config.database.postgres.db_name.is_empty() {
        errors.push(Diagnostic::new(
            "[database.postgres].db_name",
            "db_name is empty",
            "the Postgres backend needs a target database name to connect.",
            "set db_name to the database you want xrat to use.",
        ));
    }
    if config.database.postgres.max_connections == 0 {
        errors.push(Diagnostic::new(
            "[database.postgres].max_connections",
            "value is 0",
            "the connection pool needs at least one connection to work.",
            "set max_connections to 1 or more.",
        ));
    }
    if config.database.postgres.min_connections > config.database.postgres.max_connections {
        errors.push(Diagnostic::new(
            "[database.postgres].min_connections",
            format!(
                "min_connections ({}) exceeds max_connections ({})",
                config.database.postgres.min_connections, config.database.postgres.max_connections
            ),
            "the pool cannot keep more idle connections than its maximum.",
            "set min_connections less than or equal to max_connections.",
        ));
    }
    if config.database.postgres.connect_timeout_secs == 0 {
        errors.push(Diagnostic::new(
            "[database.postgres].connect_timeout_secs",
            "value is 0",
            "a 0-second connect timeout would fail immediately on a slow network.",
            "set connect_timeout_secs to a positive number of seconds.",
        ));
    }
}

pub(crate) fn validate_testing(testing: &TestingSettings, errors: &mut Vec<Diagnostic>) {
    if testing.concurrency < 0 {
        errors.push(Diagnostic::new(
            "[testing].concurrency",
            format!("value is {}", testing.concurrency),
            "concurrency cannot be negative; 0 means an automatic default.",
            "use 0 for the default, or a positive number of parallel tests.",
        ));
    }

    let mut seen = BTreeSet::new();
    for stage in &testing.order {
        if !seen.insert(test_stage_name(*stage)) {
            errors.push(Diagnostic::new(
                "[testing].order",
                format!("duplicate stage: {}", test_stage_name(*stage)),
                "each stage runs once, so duplicates have no effect and likely signal a mistake.",
                "list each stage at most once.",
            ));
        }
    }

    if let Some(codes) = &testing.real_delay.accepted_status_codes {
        for code in codes {
            if !(100..=599).contains(code) {
                errors.push(Diagnostic::new(
                    "[testing.real_delay].accepted_status_codes",
                    format!("status code {code} is outside 100-599"),
                    "HTTP status acceptance is limited to standard three-digit response classes.",
                    "use status codes from 100 through 599.",
                ));
            }
        }
    }
    if (testing.real_delay.accepted_status_codes.is_some()
        || testing.real_delay.accepted_status_ranges.is_some())
        && testing
            .real_delay
            .accepted_status_codes
            .as_ref()
            .is_none_or(Vec::is_empty)
        && testing
            .real_delay
            .accepted_status_ranges
            .as_ref()
            .is_none_or(Vec::is_empty)
    {
        errors.push(Diagnostic::new(
            "[testing.real_delay]",
            "configured status acceptance set is empty",
            "a real-delay request could never pass without an accepted status.",
            "add at least one accepted status code or range, or omit both fields to accept 200-299.",
        ));
    }

    if testing.real_delay.enabled {
        validate_http_url("[testing.real_delay].url", &testing.real_delay.url, errors);
        validate_positive(
            "[testing.real_delay].timeout",
            testing.real_delay.timeout,
            errors,
        );
    }
    if testing.download.enabled {
        validate_http_url("[testing.download].url", &testing.download.url, errors);
        validate_positive(
            "[testing.download].timeout",
            testing.download.timeout,
            errors,
        );
    }
    if testing.icmp.enabled {
        if testing.icmp.attempts == 0 {
            errors.push(Diagnostic::new(
                "[testing.icmp].attempts",
                "value is 0",
                "an ICMP test with zero attempts can never produce a result.",
                "set attempts to 1 or more.",
            ));
        }
        validate_positive("[testing.icmp].timeout", testing.icmp.timeout, errors);
    }
    if testing.tcp.enabled {
        validate_positive("[testing.tcp].timeout", testing.tcp.timeout, errors);
    }
    if testing.geoip.remote.timeout_ms == 0 {
        errors.push(Diagnostic::new(
            "[testing.geoip.remote].timeout_ms",
            "value is 0",
            "a 0-millisecond timeout would abort the remote GeoIP lookup immediately.",
            "set timeout_ms to a positive number of milliseconds.",
        ));
    }
}

pub(crate) fn validate_server(config: &AppConfig, errors: &mut Vec<Diagnostic>) {
    if !config.server.enabled {
        return;
    }

    if config.server.host.is_empty() {
        errors.push(Diagnostic::new(
            "[server].host",
            "host is empty",
            "the HTTP server needs a bind address when enabled.",
            "set host to a bind address such as 127.0.0.1 or 0.0.0.0.",
        ));
    }
    validate_secret_if_present("[server].key", config.server.key.as_ref(), errors);
}

pub(crate) fn validate_inbound(
    label: &str,
    enabled: bool,
    host: &str,
    port: u16,
    enabled_ports: &mut BTreeSet<u16>,
    errors: &mut Vec<Diagnostic>,
) {
    if !enabled {
        return;
    }

    if host.is_empty() {
        errors.push(Diagnostic::new(
            format!("{label}.host"),
            "host is empty",
            "an enabled inbound needs a bind address to listen on.",
            "set host to a bind address such as 127.0.0.1 or 0.0.0.0.",
        ));
    }
    if port == 0 {
        errors.push(Diagnostic::new(
            format!("{label}.port"),
            "port is 0",
            "0 is not a bindable TCP port.",
            "use a port in 1-65535, unique across enabled inbounds.",
        ));
    } else if !enabled_ports.insert(port) {
        errors.push(Diagnostic::new(
            format!("{label}.port"),
            format!("port {port} is already used by another enabled inbound"),
            "two inbounds cannot bind the same port at once.",
            "give each enabled inbound a distinct port in 1-65535.",
        ));
    }
}

/// Structural secret validation for the default lint path: a literal must not be
/// empty and an env reference must name a variable, but the env variable is not
/// required to be set at validation time (resolution is deferred to runtime).
pub(crate) fn validate_secret(
    label: &str,
    secret: Option<&SecretString>,
    errors: &mut Vec<Diagnostic>,
) {
    let Some(secret) = secret else {
        errors.push(Diagnostic::new(
            label,
            "secret is missing",
            "this secret is required for the enabled feature.",
            "set a literal value, or reference an environment variable with { env = \"VAR_NAME\" }.",
        ));
        return;
    };

    match secret {
        SecretString::Literal(value) if value.is_empty() => {
            errors.push(Diagnostic::new(
                label,
                "secret is empty",
                "an empty secret offers no protection and may be rejected at runtime.",
                "set a non-empty value, or reference an environment variable with { env = \"VAR_NAME\" }.",
            ));
        }
        SecretString::Literal(_) => {}
        SecretString::Env { env } if env.trim().is_empty() => {
            errors.push(Diagnostic::new(
                label,
                "env reference names no variable",
                "an env secret must name the environment variable to read at runtime.",
                "set the env field to a variable name, e.g. { env = \"XRAT_SECRET\" }.",
            ));
        }
        SecretString::Env { .. } => {}
    }
}

pub(crate) fn validate_secret_if_present(
    label: &str,
    secret: Option<&SecretString>,
    errors: &mut Vec<Diagnostic>,
) {
    if secret.is_some() {
        validate_secret(label, secret, errors);
    }
}

pub(crate) fn validate_http_url(label: &str, value: &str, errors: &mut Vec<Diagnostic>) {
    let Ok(url) = Url::parse(value) else {
        errors.push(Diagnostic::new(
            label,
            format!("not a valid URL: {value}"),
            "this test fetches the URL, so it must be parseable.",
            "use an absolute http or https URL such as https://example.com.",
        ));
        return;
    };

    if !matches!(url.scheme(), "http" | "https") {
        errors.push(Diagnostic::new(
            label,
            format!("unsupported scheme: {}", url.scheme()),
            "the tester only speaks HTTP(S) for this check.",
            "use an http or https URL.",
        ));
    }
}

pub(crate) fn validate_positive(label: &str, value: u64, errors: &mut Vec<Diagnostic>) {
    if value == 0 {
        errors.push(Diagnostic::new(
            label,
            "value is 0",
            "a 0-second timeout would abort the test before it can complete.",
            "set a positive number of seconds.",
        ));
    }
}

pub(crate) fn test_stage_name(stage: ConnectionTestStage) -> &'static str {
    stage.config_name()
}
