use crate::app::commands::output;
use crate::app::config::{AppConfig, GeoIpBackend};
use crate::app::context::AppContext;
use crate::cli::GeoIpBackendArgs;

pub(crate) fn run(context: &AppContext, args: &GeoIpBackendArgs) -> crate::app::Result<()> {
    let config =
        override_backend_config(&context.app_config, args.backend.as_deref(), args.no_cache)?;

    if args.json {
        println!("{}", format_backend_json(&config)?);
    } else {
        println!("{}", format_backend_human(&config, output::color_enabled()));
    }

    Ok(())
}

fn format_backend_human(config: &AppConfig, color: bool) -> String {
    let geoip = &config.testing.geoip;
    let mut lines = Vec::new();
    lines.push(output::format_kv(
        Some("GeoIP lookup"),
        &[
            ("enabled", output::bool_label(geoip.enabled).to_string()),
            ("backend", backend_label(geoip.backend).to_string()),
            ("fallback", backend_label(geoip.fallback).to_string()),
        ],
        color,
    ));
    lines.push(String::new());
    lines.push(output::format_kv(
        Some("Cache"),
        &[
            (
                "enabled",
                output::bool_label(geoip.cache.enabled).to_string(),
            ),
            ("ttl", format!("{} seconds", geoip.cache.ttl_secs)),
            ("max entries", geoip.cache.max_entries.to_string()),
        ],
        color,
    ));
    lines.push(String::new());
    lines.push(output::format_kv(
        Some("Remote provider"),
        &[
            (
                "provider",
                remote_provider_label(geoip.remote.provider).to_string(),
            ),
            ("endpoint", endpoint_label(&geoip.remote.endpoint)),
            ("timeout", format!("{} ms", geoip.remote.timeout_ms)),
            (
                "rate limit",
                format!("{} requests/minute", geoip.remote.rate_limit_per_minute),
            ),
        ],
        color,
    ));
    lines.push(String::new());
    lines.push(output::format_kv(
        Some("Local MMDB"),
        &[
            ("country path", geoip.country_path.display().to_string()),
            ("city path", geoip.city_path.display().to_string()),
            ("asn path", geoip.asn_path.display().to_string()),
        ],
        color,
    ));
    lines.join("\n")
}

fn format_backend_json(config: &AppConfig) -> crate::app::Result<String> {
    let geoip = &config.testing.geoip;
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "lookup": {
            "enabled": geoip.enabled,
            "backend": backend_label(geoip.backend),
            "fallback": backend_label(geoip.fallback),
        },
        "cache": {
            "enabled": geoip.cache.enabled,
            "ttl_secs": geoip.cache.ttl_secs,
            "max_entries": geoip.cache.max_entries,
        },
        "remote": {
            "provider": remote_provider_label(geoip.remote.provider),
            "endpoint": if geoip.remote.endpoint.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::Value::String(geoip.remote.endpoint.clone())
            },
            "timeout_ms": geoip.remote.timeout_ms,
            "rate_limit_per_minute": geoip.remote.rate_limit_per_minute,
        },
        "local_mmdb": {
            "country_path": geoip.country_path,
            "city_path": geoip.city_path,
            "asn_path": geoip.asn_path,
        },
    }))?)
}

fn endpoint_label(endpoint: &str) -> String {
    if endpoint.is_empty() {
        "<default>".to_string()
    } else {
        endpoint.to_string()
    }
}

pub(crate) fn override_backend_config(
    app_config: &AppConfig,
    backend_override: Option<&str>,
    no_cache: bool,
) -> crate::app::Result<AppConfig> {
    let mut config = app_config.clone();
    if let Some(backend) = backend_override {
        config.testing.geoip.backend = parse_backend_override(backend)?;
        if config.testing.geoip.backend != GeoIpBackend::Chain {
            config.testing.geoip.fallback = GeoIpBackend::None;
        }
    }
    if no_cache {
        config.testing.geoip.cache.enabled = false;
    }
    Ok(config)
}

pub(crate) fn parse_backend_override(value: &str) -> crate::app::Result<GeoIpBackend> {
    match value.trim().to_ascii_lowercase().as_str() {
        "mmdb" => Ok(GeoIpBackend::Mmdb),
        "ipwhois" | "ip-whois" => Ok(GeoIpBackend::IpWhois),
        "ip-api" | "ipapi" => Ok(GeoIpBackend::IpApi),
        "chain" => Ok(GeoIpBackend::Chain),
        other => Err(crate::app::AppError::InvalidArgument(format!(
            "unsupported geoip backend '{other}'; valid values: mmdb, ipwhois, ip-api, chain"
        ))),
    }
}

pub(crate) fn backend_label(backend: GeoIpBackend) -> &'static str {
    match backend {
        GeoIpBackend::Mmdb => "mmdb",
        GeoIpBackend::IpWhois => "ipwhois",
        GeoIpBackend::IpApi => "ip-api",
        GeoIpBackend::Chain => "chain",
        GeoIpBackend::None => "none",
    }
}

pub(crate) fn remote_provider_label(
    provider: crate::app::config::GeoIpRemoteProvider,
) -> &'static str {
    match provider {
        crate::app::config::GeoIpRemoteProvider::IpWhois => "ipwhois",
        crate::app::config::GeoIpRemoteProvider::IpApi => "ip-api",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overrides_backend_and_disables_cache() {
        let config = override_backend_config(&AppConfig::default(), Some("ip-api"), true).unwrap();

        assert_eq!(config.testing.geoip.backend, GeoIpBackend::IpApi);
        assert!(!config.testing.geoip.cache.enabled);
    }

    #[test]
    fn formats_backend_output_in_sections() {
        let output = format_backend_human(&AppConfig::default(), false);

        assert!(output.contains("GeoIP lookup"));
        assert!(output.contains("Cache"));
        assert!(output.contains("Remote provider"));
        assert!(output.contains("Local MMDB"));
        assert!(output.contains("ttl          86400 seconds"));
        assert!(output.contains("timeout     5000 ms"));
    }

    #[test]
    fn formats_backend_json() {
        let output = format_backend_json(&AppConfig::default()).unwrap();
        let json: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(json["lookup"]["backend"], "mmdb");
        assert_eq!(json["cache"]["enabled"], true);
        assert_eq!(json["remote"]["endpoint"], serde_json::Value::Null);
        assert_eq!(
            json["local_mmdb"]["country_path"],
            "mmdb/GeoLite2-Country.mmdb"
        );
    }
}
