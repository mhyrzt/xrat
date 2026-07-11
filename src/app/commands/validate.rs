use std::collections::BTreeSet;

use serde::Serialize;
use url::Url;

use crate::app::AppError;
use crate::app::commands::output::{self, Style};
use crate::app::config::{
    AppConfig, ConnectionTestStage, DatabaseBackend, SecretString, TestingSettings,
};
use crate::cli::{ValidateArgs, ValidateFormat};

/// One validation finding. Beyond identifying the offending field, each
/// diagnostic explains why the value is rejected and how to repair it so users
/// can fix `config.toml` without reading source or guessing valid ranges.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
struct Diagnostic {
    /// Config path such as `[runtime.socks].port`. Empty for structural or
    /// parse failures that are not tied to a single field.
    field: String,
    /// What is invalid about the current value.
    problem: String,
    /// Why the constraint matters.
    reason: String,
    /// How to fix it, including accepted values or ranges where relevant.
    fix: String,
}

impl Diagnostic {
    fn new(
        field: impl Into<String>,
        problem: impl Into<String>,
        reason: impl Into<String>,
        fix: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            problem: problem.into(),
            reason: reason.into(),
            fix: fix.into(),
        }
    }

    /// A finding not tied to a specific config field (missing file, parse
    /// failure). Rendered without a leading field label.
    fn structural(
        problem: impl Into<String>,
        reason: impl Into<String>,
        fix: impl Into<String>,
    ) -> Self {
        Self::new(String::new(), problem, reason, fix)
    }
}

pub fn run(args: &ValidateArgs) -> crate::app::Result<()> {
    let diagnostics = collect_errors(args);
    let color = output::color_enabled();

    match args.format {
        ValidateFormat::Human => {
            if diagnostics.is_empty() {
                println!(
                    "{}",
                    output::success(format!("{} is valid.", args.path.display()), color)
                );
            } else {
                eprintln!("{}", render_human(args, &diagnostics, color));
            }
        }
        ValidateFormat::Json => {
            let report = serde_json::json!({
                "path": args.path.display().to_string(),
                "valid": diagnostics.is_empty(),
                "errors": diagnostics,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
            );
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(AppError::InvalidArgument(format!(
            "{} is invalid",
            args.path.display()
        )))
    }
}

fn render_human(args: &ValidateArgs, diagnostics: &[Diagnostic], color: bool) -> String {
    let title = format!(
        "{} has {} validation error(s):",
        args.path.display(),
        diagnostics.len()
    );

    let mut lines = Vec::with_capacity(diagnostics.len() * 4 + 1);
    lines.push(output::style_text(&title, Style::Dim, color));

    for diagnostic in diagnostics {
        lines.push(String::new());
        let heading = if diagnostic.field.is_empty() {
            diagnostic.problem.clone()
        } else {
            format!(
                "{} {}",
                output::style_text(&diagnostic.field, Style::Red, color),
                diagnostic.problem
            )
        };
        lines.push(format!("  {heading}"));
        lines.push(format!(
            "    {} {}",
            output::style_text("why:", Style::Dim, color),
            diagnostic.reason
        ));
        lines.push(format!(
            "    {} {}",
            output::style_text("fix:", Style::Dim, color),
            diagnostic.fix
        ));
    }

    lines.join("\n")
}

/// Collect all validation findings. Structural problems (missing/non-file path,
/// parse failure) short-circuit deeper checks since there is no config to lint.
fn collect_errors(args: &ValidateArgs) -> Vec<Diagnostic> {
    let mut errors = Vec::new();

    if !args.path.exists() {
        errors.push(Diagnostic::structural(
            format!("config file does not exist: {}", args.path.display()),
            "validation needs an existing config file to read.",
            "create the file or pass the correct path; run `xrat init` to generate a default config.",
        ));
        return errors;
    }
    if !args.path.is_file() {
        errors.push(Diagnostic::structural(
            format!("config path is not a file: {}", args.path.display()),
            "the path points at a directory or special file, not a readable config.",
            "pass the path to the config.toml file itself.",
        ));
        return errors;
    }

    let contents = match std::fs::read_to_string(&args.path) {
        Ok(contents) => contents,
        Err(err) => {
            errors.push(Diagnostic::structural(
                format!("config file could not be read: {err}"),
                "the file exists but could not be opened for reading.",
                "check file permissions and that the path points at a regular file.",
            ));
            return errors;
        }
    };

    match toml::from_str::<toml::Value>(&contents) {
        Ok(value) => check_known_fields(&value, &mut errors),
        Err(err) => {
            errors.push(Diagnostic::structural(
                format!("config file is not valid TOML: {err}"),
                "the file could not be parsed as TOML syntax.",
                "fix the reported syntax error, including quoting, commas, and table headers.",
            ));
            return errors;
        }
    }

    if !errors.is_empty() {
        return errors;
    }

    match crate::app::config::load(&args.path) {
        Ok(config) => validate_config(&config, &mut errors),
        Err(err) => errors.push(Diagnostic::structural(
            format!("config failed to parse: {err}"),
            "the file is not valid TOML or has a field with the wrong type or an unknown enum value.",
            "fix the reported syntax or value; check quoting, table headers, and that enum fields use accepted values.",
        )),
    }

    errors
}

const TEST_STAGE_ENUM: &[(&str, &[&str])] = &[
    ("icmp", &["ping"]),
    ("real_delay", &["real-delay"]),
    ("download", &["download-speed"]),
    ("tcp", &[]),
];

const FAILURE_POLICY_ENUM: &[(&str, &[&str])] = &[
    ("continue", &[]),
    ("skip_remaining", &["skip-remaining"]),
    ("mark_failed", &["mark-failed"]),
];

const DATABASE_BACKEND_ENUM: &[(&str, &[&str])] = &[("sqlite", &[]), ("postgres", &[])];

const GEOIP_BACKEND_ENUM: &[(&str, &[&str])] = &[
    ("mmdb", &[]),
    ("ip-whois", &["ipwhois"]),
    ("ip-api", &["ipapi"]),
    ("chain", &[]),
    ("none", &[]),
];

const GEOIP_PROVIDER_ENUM: &[(&str, &[&str])] =
    &[("ip-whois", &["ipwhois"]), ("ip-api", &["ipapi"])];

/// Field-level checks against the raw TOML value, run before deserializing into
/// `AppConfig`. These catch invalid enum values and wrong types on fields that
/// would otherwise fail deserialization with only a generic parse error.
fn check_known_fields(value: &toml::Value, errors: &mut Vec<Diagnostic>) {
    check_string_array_enum(
        value,
        &["testing", "order"],
        "[testing].order",
        TEST_STAGE_ENUM,
        errors,
    );
    check_scalar_enum(
        value,
        &["testing", "failure_policy"],
        "[testing].failure_policy",
        FAILURE_POLICY_ENUM,
        errors,
    );
    check_duration_field(
        value,
        &["testing", "tcp", "timeout"],
        "[testing.tcp].timeout",
        errors,
    );
    check_scalar_enum(
        value,
        &["database", "backend"],
        "[database].backend",
        DATABASE_BACKEND_ENUM,
        errors,
    );
    check_scalar_enum(
        value,
        &["testing", "geoip", "backend"],
        "[testing.geoip].backend",
        GEOIP_BACKEND_ENUM,
        errors,
    );
    check_scalar_enum(
        value,
        &["testing", "geoip", "fallback"],
        "[testing.geoip].fallback",
        GEOIP_BACKEND_ENUM,
        errors,
    );
    check_scalar_enum(
        value,
        &["testing", "geoip", "remote", "provider"],
        "[testing.geoip.remote].provider",
        GEOIP_PROVIDER_ENUM,
        errors,
    );
}

fn get_path<'a>(value: &'a toml::Value, path: &[&str]) -> Option<&'a toml::Value> {
    let mut current = value;
    for segment in path {
        current = current.as_table()?.get(*segment)?;
    }
    Some(current)
}

fn toml_kind(value: &toml::Value) -> &'static str {
    match value {
        toml::Value::String(_) => "a string",
        toml::Value::Integer(_) => "an integer",
        toml::Value::Float(_) => "a float",
        toml::Value::Boolean(_) => "a boolean",
        toml::Value::Datetime(_) => "a datetime",
        toml::Value::Array(_) => "an array",
        toml::Value::Table(_) => "a table",
    }
}

fn enum_value_matches(accepted: &[(&str, &[&str])], text: &str) -> bool {
    accepted
        .iter()
        .any(|(canonical, aliases)| *canonical == text || aliases.contains(&text))
}

fn accepted_values_list(accepted: &[(&str, &[&str])]) -> String {
    accepted
        .iter()
        .map(|(canonical, aliases)| {
            if aliases.is_empty() {
                canonical.to_string()
            } else {
                format!("{canonical} (alias {})", aliases.join(", "))
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn check_scalar_enum(
    value: &toml::Value,
    path: &[&str],
    field: &str,
    accepted: &[(&str, &[&str])],
    errors: &mut Vec<Diagnostic>,
) {
    let Some(found) = get_path(value, path) else {
        return;
    };
    let Some(text) = found.as_str() else {
        errors.push(Diagnostic::new(
            field,
            format!("expected a string, got {}", toml_kind(found)),
            "this field selects one of a fixed set of named options.",
            format!("use one of: {}", accepted_values_list(accepted)),
        ));
        return;
    };
    if !enum_value_matches(accepted, text) {
        errors.push(Diagnostic::new(
            field,
            format!("value \"{text}\" is not accepted here"),
            "this field selects one of a fixed set of named options.",
            format!("accepted values: {}", accepted_values_list(accepted)),
        ));
    }
}

fn check_string_array_enum(
    value: &toml::Value,
    path: &[&str],
    field: &str,
    accepted: &[(&str, &[&str])],
    errors: &mut Vec<Diagnostic>,
) {
    let Some(found) = get_path(value, path) else {
        return;
    };
    let Some(array) = found.as_array() else {
        errors.push(Diagnostic::new(
            field,
            format!("expected an array of strings, got {}", toml_kind(found)),
            "this field lists stages to run.",
            format!("use an array such as [\"{}\"]", accepted[0].0),
        ));
        return;
    };
    for entry in array {
        let Some(text) = entry.as_str() else {
            errors.push(Diagnostic::new(
                field,
                format!("array entry is not a string: {entry}"),
                "each entry names a stage.",
                format!("accepted values: {}", accepted_values_list(accepted)),
            ));
            continue;
        };
        if !enum_value_matches(accepted, text) {
            errors.push(Diagnostic::new(
                field,
                format!("value \"{text}\" is not accepted here"),
                "each entry must name a known stage.",
                format!("accepted values: {}", accepted_values_list(accepted)),
            ));
        }
    }
}

fn check_duration_field(
    value: &toml::Value,
    path: &[&str],
    field: &str,
    errors: &mut Vec<Diagnostic>,
) {
    let Some(found) = get_path(value, path) else {
        return;
    };
    match found {
        toml::Value::Integer(number) if *number < 0 => {
            errors.push(Diagnostic::new(
                field,
                format!("value is {number}"),
                "a duration in milliseconds cannot be negative.",
                "use 0 or a positive number of milliseconds.",
            ));
        }
        toml::Value::Integer(_) => {}
        other => {
            errors.push(Diagnostic::new(
                field,
                format!("expected integer milliseconds, got {}", toml_kind(other)),
                "this field is a duration in milliseconds.",
                "use a plain integer, e.g. 2000.",
            ));
        }
    }
}

fn validate_config(config: &AppConfig, errors: &mut Vec<Diagnostic>) {
    validate_runtime(config, errors);
    validate_database(config, errors);
    validate_testing(&config.testing, errors);
    validate_server(config, errors);
}

fn validate_runtime(config: &AppConfig, errors: &mut Vec<Diagnostic>) {
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
}

fn validate_mux(mux: &crate::app::config::MuxSettings, errors: &mut Vec<Diagnostic>) {
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

fn validate_fragment(
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

fn validate_network(network: &crate::app::config::NetworkSettings, errors: &mut Vec<Diagnostic>) {
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

fn validate_database(config: &AppConfig, errors: &mut Vec<Diagnostic>) {
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

fn validate_testing(testing: &TestingSettings, errors: &mut Vec<Diagnostic>) {
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

fn validate_server(config: &AppConfig, errors: &mut Vec<Diagnostic>) {
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

fn validate_inbound(
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
fn validate_secret(label: &str, secret: Option<&SecretString>, errors: &mut Vec<Diagnostic>) {
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

fn validate_secret_if_present(
    label: &str,
    secret: Option<&SecretString>,
    errors: &mut Vec<Diagnostic>,
) {
    if secret.is_some() {
        validate_secret(label, secret, errors);
    }
}

fn validate_http_url(label: &str, value: &str, errors: &mut Vec<Diagnostic>) {
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

fn validate_positive(label: &str, value: u64, errors: &mut Vec<Diagnostic>) {
    if value == 0 {
        errors.push(Diagnostic::new(
            label,
            "value is 0",
            "a 0-second timeout would abort the test before it can complete.",
            "set a positive number of seconds.",
        ));
    }
}

fn test_stage_name(stage: ConnectionTestStage) -> &'static str {
    stage.config_name()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn errors_for(config: &AppConfig) -> Vec<Diagnostic> {
        let mut errors = Vec::new();
        validate_config(config, &mut errors);
        errors
    }

    fn find<'a>(diagnostics: &'a [Diagnostic], field: &str) -> &'a Diagnostic {
        diagnostics
            .iter()
            .find(|diagnostic| diagnostic.field == field)
            .unwrap_or_else(|| panic!("expected a diagnostic for {field}: {diagnostics:?}"))
    }

    fn errors_for_toml(contents: &str) -> Vec<Diagnostic> {
        let root_dir = std::env::temp_dir().join(format!(
            "xrat-validate-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root_dir).expect("temp dir should be created");
        let config_path = root_dir.join("config.toml");
        std::fs::write(&config_path, contents).expect("config should be written");

        let errors = collect_errors(&ValidateArgs {
            path: config_path.clone(),
            format: ValidateFormat::Human,
        });

        let _ = std::fs::remove_file(config_path);
        let _ = std::fs::remove_dir(root_dir);
        errors
    }

    #[test]
    fn accepts_default_config() {
        let errors = errors_for(&AppConfig::default());

        assert!(
            errors.is_empty(),
            "default config should validate: {errors:?}"
        );
    }

    #[test]
    fn rejects_invalid_runtime_engine() {
        let mut config = AppConfig::default();
        config.runtime.engine = "bad".to_string();

        let errors = errors_for(&config);
        let diagnostic = find(&errors, "[runtime].engine");

        assert!(diagnostic.problem.contains("bad"));
        assert!(diagnostic.fix.contains("xray"));
        assert!(!diagnostic.reason.is_empty());
    }

    #[test]
    fn rejects_invalid_mux_values() {
        let mut config = AppConfig::default();
        config.runtime.mux.concurrency = 200;
        config.runtime.mux.xudp_concurrency = 5000;
        config.runtime.mux.xudp_proxy_udp443 = "nope".to_string();

        let errors = errors_for(&config);
        assert!(
            find(&errors, "[runtime.mux].concurrency")
                .fix
                .contains("128")
        );
        assert!(
            find(&errors, "[runtime.mux].xudp_concurrency")
                .fix
                .contains("1024")
        );
        assert!(
            find(&errors, "[runtime.mux].xudp_proxy_udp443")
                .fix
                .contains("reject")
        );
    }

    #[test]
    fn rejects_invalid_fragment_ranges() {
        let mut config = AppConfig::default();
        config.runtime.fragment.packets_mode = "range".to_string();
        config.runtime.fragment.packets = [0, 3];
        config.runtime.fragment.length = [200, 100];
        config.runtime.fragment.interval = [30, 10];

        let errors = errors_for(&config);
        assert!(
            !find(&errors, "[runtime.fragment].packets")
                .reason
                .is_empty()
        );
        assert!(!find(&errors, "[runtime.fragment].length").reason.is_empty());
        assert!(
            !find(&errors, "[runtime.fragment].interval")
                .reason
                .is_empty()
        );
    }

    #[test]
    fn rejects_invalid_network_bind_address() {
        let mut config = AppConfig::default();
        config.runtime.network.bind_address = "not-an-ip".to_string();
        config.runtime.network.mark = -1;

        let errors = errors_for(&config);
        assert!(
            find(&errors, "[runtime.network].bind_address")
                .problem
                .contains("not-an-ip")
        );
        assert!(!find(&errors, "[runtime.network].mark").reason.is_empty());
    }

    #[test]
    fn rejects_unknown_fragment_packets_mode() {
        let mut config = AppConfig::default();
        config.runtime.fragment.packets_mode = "bogus".to_string();

        let errors = errors_for(&config);
        assert!(
            find(&errors, "[runtime.fragment].packets_mode")
                .fix
                .contains("range")
        );
    }

    #[test]
    fn accepts_valid_fragment_packet_range() {
        let mut config = AppConfig::default();
        config.runtime.fragment.packets_mode = "range".to_string();
        config.runtime.fragment.packets = [1, 3];

        let errors = errors_for(&config);
        assert!(
            !errors
                .iter()
                .any(|diagnostic| diagnostic.field == "[runtime.fragment].packets"),
            "valid packets range should not error: {errors:?}"
        );
    }

    #[test]
    fn rejects_duplicate_enabled_inbound_ports() {
        let mut config = AppConfig::default();
        config.runtime.http.enabled = true;
        config.runtime.http.port = config.runtime.socks.port;

        let errors = errors_for(&config);
        let diagnostic = find(&errors, "[runtime.http].port");

        assert!(diagnostic.problem.contains("already used"));
        assert!(diagnostic.fix.contains("1-65535"));
    }

    #[test]
    fn port_zero_reports_range_in_fix() {
        let mut config = AppConfig::default();
        config.runtime.socks.port = 0;

        let errors = errors_for(&config);
        let diagnostic = find(&errors, "[runtime.socks].port");

        assert!(diagnostic.fix.contains("1-65535"));
        assert!(!diagnostic.reason.is_empty());
    }

    #[test]
    fn timeout_zero_reports_reason_and_fix() {
        let mut config = AppConfig::default();
        config.testing.tcp.enabled = true;
        config.testing.tcp.timeout = 0;

        let errors = errors_for(&config);
        let diagnostic = find(&errors, "[testing.tcp].timeout");

        assert!(!diagnostic.reason.is_empty());
        assert!(diagnostic.fix.contains("seconds"));
    }

    #[test]
    fn invalid_test_url_reports_scheme_guidance() {
        let mut config = AppConfig::default();
        config.testing.download.enabled = true;
        config.testing.download.url = "ftp://example.com".to_string();

        let errors = errors_for(&config);
        let diagnostic = find(&errors, "[testing.download].url");

        assert!(diagnostic.fix.contains("http"));
    }

    #[test]
    fn accepts_rotation_stage_aliases() {
        let mut config = AppConfig::default();
        config.runtime.rotation.test_stages = vec![
            "ping".to_string(),
            "real-delay".to_string(),
            "download".to_string(),
        ];

        let errors = errors_for(&config);

        assert!(
            !errors
                .iter()
                .any(|diagnostic| diagnostic.field == "[runtime.rotation].test_stages"),
            "stage aliases should be accepted: {errors:?}"
        );
    }

    #[test]
    fn rejects_unknown_rotation_stage() {
        let mut config = AppConfig::default();
        config.runtime.rotation.test_stages = vec!["bogus".to_string()];

        let errors = errors_for(&config);
        let diagnostic = find(&errors, "[runtime.rotation].test_stages");

        assert!(diagnostic.problem.contains("bogus"));
        assert!(diagnostic.fix.contains("icmp"));
    }

    #[test]
    fn env_secret_does_not_require_resolution() {
        let mut config = AppConfig::default();
        config.server.enabled = true;
        config.server.key = Some(SecretString::Env {
            env: "XRAT_UNSET_VALIDATE_KEY".to_string(),
        });

        let errors = errors_for(&config);

        assert!(
            !errors
                .iter()
                .any(|diagnostic| diagnostic.field == "[server].key"),
            "env-referenced secret should pass structural validation: {errors:?}"
        );
    }

    #[test]
    fn rejects_missing_config_file() {
        let args = ValidateArgs {
            path: "/tmp/xrat-missing-config.toml".into(),
            format: ValidateFormat::Human,
        };

        let errors = collect_errors(&args);

        assert!(
            errors
                .iter()
                .any(|diagnostic| diagnostic.problem.contains("config file does not exist"))
        );
    }

    #[test]
    fn rejects_unknown_test_stage_before_deserializing() {
        let errors = errors_for_toml(
            r#"
            [testing]
            order = ["icmp", "bogus"]
            "#,
        );

        let diagnostic = find(&errors, "[testing].order");
        assert!(diagnostic.problem.contains("bogus"));
        assert!(diagnostic.fix.contains("icmp"));
        assert!(diagnostic.fix.contains("tcp"));
    }

    #[test]
    fn rejects_unknown_failure_policy_before_deserializing() {
        let errors = errors_for_toml(
            r#"
            [testing]
            failure_policy = "skip"
            "#,
        );

        let diagnostic = find(&errors, "[testing].failure_policy");
        assert!(diagnostic.problem.contains("skip"));
        assert!(diagnostic.fix.contains("skip_remaining"));
        assert!(diagnostic.fix.contains("alias"));
    }

    #[test]
    fn rejects_string_typed_duration_before_deserializing() {
        let errors = errors_for_toml(
            r#"
            [testing.tcp]
            timeout = "2000"
            "#,
        );

        let diagnostic = find(&errors, "[testing.tcp].timeout");
        assert!(diagnostic.problem.contains("expected integer milliseconds"));
        assert!(diagnostic.problem.contains("a string"));
    }

    #[test]
    fn rejects_unknown_database_backend_before_deserializing() {
        let errors = errors_for_toml(
            r#"
            [database]
            backend = "mysql"
            "#,
        );

        let diagnostic = find(&errors, "[database].backend");
        assert!(diagnostic.problem.contains("mysql"));
        assert!(diagnostic.fix.contains("postgres"));
    }

    #[test]
    fn rejects_unknown_geoip_backend_before_deserializing() {
        let errors = errors_for_toml(
            r#"
            [testing.geoip]
            backend = "bogus"
            "#,
        );

        let diagnostic = find(&errors, "[testing.geoip].backend");
        assert!(diagnostic.problem.contains("bogus"));
        assert!(diagnostic.fix.contains("mmdb"));
    }

    #[test]
    fn reports_invalid_toml_syntax_as_structural_error() {
        let errors = errors_for_toml("[testing\norder = [\"icmp\"]\n");

        assert!(
            errors
                .iter()
                .any(|diagnostic| diagnostic.problem.contains("not valid TOML")),
            "expected a syntax diagnostic: {errors:?}"
        );
    }

    #[test]
    fn accepts_valid_config_written_to_disk() {
        let errors = errors_for_toml(
            r#"
            [testing]
            order = ["icmp", "tcp", "real_delay"]
            failure_policy = "skip_remaining"

            [testing.tcp]
            enabled = true
            timeout = 2000
            "#,
        );

        assert!(
            errors.is_empty(),
            "valid config should have no errors: {errors:?}"
        );
    }
}
