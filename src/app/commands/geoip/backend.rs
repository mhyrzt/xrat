use crate::app::config::{AppConfig, GeoIpBackend};
use crate::app::context::AppContext;
use crate::cli::GeoIpBackendArgs;

pub(crate) fn run(context: &AppContext, args: &GeoIpBackendArgs) -> crate::app::Result<()> {
    let config =
        override_backend_config(&context.app_config, args.backend.as_deref(), args.no_cache)?;

    println!("backend: {}", backend_label(config.testing.geoip.backend));
    println!("fallback: {}", backend_label(config.testing.geoip.fallback));
    println!(
        "cache: {}",
        if config.testing.geoip.cache.enabled {
            format!(
                "enabled (ttl={}s, max={})",
                config.testing.geoip.cache.ttl_secs, config.testing.geoip.cache.max_entries
            )
        } else {
            "disabled".to_string()
        }
    );
    println!(
        "remote: provider={} endpoint={} timeout_ms={} rate_limit_per_minute={}",
        remote_provider_label(config.testing.geoip.remote.provider),
        if config.testing.geoip.remote.endpoint.is_empty() {
            "<default>"
        } else {
            config.testing.geoip.remote.endpoint.as_str()
        },
        config.testing.geoip.remote.timeout_ms,
        config.testing.geoip.remote.rate_limit_per_minute,
    );

    Ok(())
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
}
