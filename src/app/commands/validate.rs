use std::collections::BTreeSet;

use url::Url;

use crate::app::AppError;
use crate::app::commands::output;
use crate::app::config::{
    AppConfig, ConnectionTestStage, DatabaseBackend, SecretString, TestingSettings,
};
use crate::cli::ValidateArgs;

pub fn run(args: &ValidateArgs) -> crate::app::Result<()> {
    validate_path(args)?;
    let config = crate::app::config::load(&args.path)?;
    validate_config(&config)?;

    println!(
        "{}",
        output::success(
            format!("{} is valid.", args.path.display()),
            output::color_enabled()
        )
    );

    Ok(())
}

fn validate_path(args: &ValidateArgs) -> crate::app::Result<()> {
    if !args.path.exists() {
        return Err(AppError::InvalidArgument(format!(
            "config file does not exist: {}",
            args.path.display()
        )));
    }

    if !args.path.is_file() {
        return Err(AppError::InvalidArgument(format!(
            "config path is not a file: {}",
            args.path.display()
        )));
    }

    Ok(())
}

fn validate_config(config: &AppConfig) -> crate::app::Result<()> {
    let mut errors = Vec::new();

    validate_runtime(config, &mut errors);
    validate_database(config, &mut errors);
    validate_testing(&config.testing, &mut errors);
    validate_server(config, &mut errors);

    if errors.is_empty() {
        return Ok(());
    }

    Err(AppError::InvalidArgument(errors.join("; ")))
}

fn validate_runtime(config: &AppConfig, errors: &mut Vec<String>) {
    let runtime = &config.runtime;
    if !matches!(runtime.engine.as_str(), "xray" | "v2ray" | "sing-box") {
        errors.push("[runtime].engine must be one of: xray, v2ray, sing-box".to_string());
    }

    if runtime.rotation.test_concurrency < 0 {
        errors.push("[runtime.rotation].test_concurrency must be 0 or greater".to_string());
    }

    for stage in &runtime.rotation.test_stages {
        if !matches!(stage.as_str(), "icmp" | "ping" | "real_delay" | "download") {
            errors.push(format!(
                "[runtime.rotation].test_stages contains unsupported stage: {stage}"
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
            errors
                .push("[runtime.socks.auth].username is required when auth is enabled".to_string());
        }
        validate_secret(
            "[runtime.socks.auth].password",
            runtime.socks.auth.password.as_ref(),
            errors,
        );
    }

    if runtime.shadowsocks.enabled {
        if runtime.shadowsocks.method.is_empty() {
            errors.push("[runtime.shadowsocks].method is required when enabled".to_string());
        }
        validate_secret(
            "[runtime.shadowsocks].password",
            Some(&runtime.shadowsocks.password),
            errors,
        );
        for network in runtime.shadowsocks.network.split(',') {
            if !matches!(network.trim(), "tcp" | "udp") {
                errors.push(format!(
                    "[runtime.shadowsocks].network contains unsupported value: {network}"
                ));
            }
        }
    }
}

fn validate_database(config: &AppConfig, errors: &mut Vec<String>) {
    if config.database.backend != DatabaseBackend::Postgres {
        return;
    }

    validate_secret(
        "[database.postgres].user",
        Some(&config.database.postgres.user),
        errors,
    );
    if config.database.postgres.db_name.is_empty() {
        errors.push(
            "[database.postgres].db_name is required when backend = \"postgres\"".to_string(),
        );
    }
    if config.database.postgres.max_connections == 0 {
        errors.push("[database.postgres].max_connections must be greater than 0".to_string());
    }
    if config.database.postgres.min_connections > config.database.postgres.max_connections {
        errors.push(
            "[database.postgres].min_connections must be less than or equal to max_connections"
                .to_string(),
        );
    }
    if config.database.postgres.connect_timeout_secs == 0 {
        errors.push("[database.postgres].connect_timeout_secs must be greater than 0".to_string());
    }
}

fn validate_testing(testing: &TestingSettings, errors: &mut Vec<String>) {
    if testing.concurrency < 0 {
        errors.push("[testing].concurrency must be 0 or greater".to_string());
    }

    let mut seen = BTreeSet::new();
    for stage in &testing.order {
        if !seen.insert(test_stage_name(*stage)) {
            errors.push(format!(
                "[testing].order contains duplicate stage: {}",
                test_stage_name(*stage)
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
            errors.push("[testing.icmp].attempts must be greater than 0".to_string());
        }
        validate_positive("[testing.icmp].timeout", testing.icmp.timeout, errors);
    }
    if testing.tcp.enabled {
        validate_positive("[testing.tcp].timeout", testing.tcp.timeout, errors);
    }
    if testing.geoip.remote.timeout_ms == 0 {
        errors.push("[testing.geoip.remote].timeout_ms must be greater than 0".to_string());
    }
}

fn validate_server(config: &AppConfig, errors: &mut Vec<String>) {
    if !config.server.enabled {
        return;
    }

    if config.server.host.is_empty() {
        errors.push("[server].host is required when enabled".to_string());
    }
    validate_secret_if_present("[server].key", config.server.key.as_ref(), errors);
}

fn validate_inbound(
    label: &str,
    enabled: bool,
    host: &str,
    port: u16,
    enabled_ports: &mut BTreeSet<u16>,
    errors: &mut Vec<String>,
) {
    if !enabled {
        return;
    }

    if host.is_empty() {
        errors.push(format!("{label}.host is required when enabled"));
    }
    if port == 0 {
        errors.push(format!("{label}.port must be greater than 0"));
    } else if !enabled_ports.insert(port) {
        errors.push(format!(
            "{label}.port duplicates another enabled inbound port: {port}"
        ));
    }
}

fn validate_secret(label: &str, secret: Option<&SecretString>, errors: &mut Vec<String>) {
    let Some(secret) = secret else {
        errors.push(format!("{label} is required"));
        return;
    };

    match secret.resolve() {
        Ok(value) if value.is_empty() => errors.push(format!("{label} must not be empty")),
        Ok(_) => {}
        Err(error) => errors.push(format!("{label} is invalid: {error}")),
    }
}

fn validate_secret_if_present(
    label: &str,
    secret: Option<&SecretString>,
    errors: &mut Vec<String>,
) {
    if secret.is_some() {
        validate_secret(label, secret, errors);
    }
}

fn validate_http_url(label: &str, value: &str, errors: &mut Vec<String>) {
    let Ok(url) = Url::parse(value) else {
        errors.push(format!("{label} must be a valid HTTP or HTTPS URL"));
        return;
    };

    if !matches!(url.scheme(), "http" | "https") {
        errors.push(format!("{label} must use http or https"));
    }
}

fn validate_positive(label: &str, value: u64, errors: &mut Vec<String>) {
    if value == 0 {
        errors.push(format!("{label} must be greater than 0"));
    }
}

fn test_stage_name(stage: ConnectionTestStage) -> &'static str {
    match stage {
        ConnectionTestStage::Icmp => "icmp",
        ConnectionTestStage::RealDelay => "real_delay",
        ConnectionTestStage::Download => "download",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_default_config() {
        let config = AppConfig::default();

        validate_config(&config).expect("default config should validate");
    }

    #[test]
    fn rejects_invalid_runtime_engine() {
        let mut config = AppConfig::default();
        config.runtime.engine = "bad".to_string();

        let error = validate_config(&config).expect_err("engine should be rejected");

        assert!(error.to_string().contains("[runtime].engine"));
    }

    #[test]
    fn rejects_duplicate_enabled_inbound_ports() {
        let mut config = AppConfig::default();
        config.runtime.http.enabled = true;
        config.runtime.http.port = config.runtime.socks.port;

        let error = validate_config(&config).expect_err("duplicate port should be rejected");

        assert!(
            error
                .to_string()
                .contains("duplicates another enabled inbound port")
        );
    }

    #[test]
    fn rejects_missing_config_file() {
        let args = ValidateArgs {
            path: "/tmp/xrat-missing-config.toml".into(),
        };

        let error = validate_path(&args).expect_err("missing file should be rejected");

        assert!(error.to_string().contains("config file does not exist"));
    }
}
