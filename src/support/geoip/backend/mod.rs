use std::sync::Arc;

use crate::app::config::{AppConfig, GeoIpBackend};
use crate::app::context::RuntimePaths;
use crate::app::paths::mmdb;

use super::{
    CachedLookup, ChainedLookup, GeoIpLookup, LocalMmdbLookup, RateLimitedLookup,
    RemoteIpApiLookup, RemoteIpWhoisLookup,
};

mod validation;

#[cfg(test)]
mod tests;

pub fn build_lookup_chain(
    app_config: &AppConfig,
    runtime_paths: &RuntimePaths,
) -> crate::app::Result<Arc<dyn GeoIpLookup>> {
    validation::validate_geoip_settings(&app_config.testing.geoip)?;

    match app_config.testing.geoip.backend {
        GeoIpBackend::Mmdb => Ok(build_mmdb_lookup(app_config, runtime_paths)),
        GeoIpBackend::IpWhois => build_remote_lookup(
            Arc::new(RemoteIpWhoisLookup::new(
                app_config.testing.geoip.remote.endpoint.clone(),
                std::time::Duration::from_millis(app_config.testing.geoip.remote.timeout_ms),
            )?),
            app_config,
        ),
        GeoIpBackend::IpApi => build_remote_lookup(
            Arc::new(RemoteIpApiLookup::new(
                app_config.testing.geoip.remote.endpoint.clone(),
                std::time::Duration::from_millis(app_config.testing.geoip.remote.timeout_ms),
            )?),
            app_config,
        ),
        GeoIpBackend::Chain => Ok(Arc::new(ChainedLookup::new(
            build_mmdb_lookup(app_config, runtime_paths),
            build_chain_fallback(app_config)?,
        ))),
        GeoIpBackend::None => Err(crate::app::AppError::InvalidArgument(
            "[testing.geoip].backend cannot be 'none'".to_string(),
        )),
    }
}

fn build_mmdb_lookup(app_config: &AppConfig, runtime_paths: &RuntimePaths) -> Arc<dyn GeoIpLookup> {
    Arc::new(LocalMmdbLookup::new(
        mmdb::mmdb_path_for(
            runtime_paths,
            app_config,
            &app_config.testing.geoip.country_path,
            "GeoLite2-Country.mmdb",
        ),
        mmdb::mmdb_path_for(
            runtime_paths,
            app_config,
            &app_config.testing.geoip.city_path,
            "GeoLite2-City.mmdb",
        ),
        mmdb::mmdb_path_for(
            runtime_paths,
            app_config,
            &app_config.testing.geoip.asn_path,
            "GeoLite2-ASN.mmdb",
        ),
    ))
}

fn build_chain_fallback(app_config: &AppConfig) -> crate::app::Result<Arc<dyn GeoIpLookup>> {
    match app_config.testing.geoip.fallback {
        GeoIpBackend::IpWhois => build_remote_lookup(
            Arc::new(RemoteIpWhoisLookup::new(
                app_config.testing.geoip.remote.endpoint.clone(),
                std::time::Duration::from_millis(app_config.testing.geoip.remote.timeout_ms),
            )?),
            app_config,
        ),
        GeoIpBackend::IpApi => build_remote_lookup(
            Arc::new(RemoteIpApiLookup::new(
                app_config.testing.geoip.remote.endpoint.clone(),
                std::time::Duration::from_millis(app_config.testing.geoip.remote.timeout_ms),
            )?),
            app_config,
        ),
        _ => Err(crate::app::AppError::InvalidArgument(
            "[testing.geoip].fallback must be ipwhois or ip-api when backend = 'chain'".to_string(),
        )),
    }
}

fn build_remote_lookup(
    remote: Arc<dyn GeoIpLookup>,
    app_config: &AppConfig,
) -> crate::app::Result<Arc<dyn GeoIpLookup>> {
    let rate_limited: Arc<dyn GeoIpLookup> = Arc::new(RateLimitedLookup::new(
        remote,
        app_config.testing.geoip.remote.rate_limit_per_minute,
        std::time::Duration::from_secs(60),
    ));
    if app_config.testing.geoip.cache.enabled {
        Ok(Arc::new(CachedLookup::new(
            rate_limited,
            std::time::Duration::from_secs(app_config.testing.geoip.cache.ttl_secs),
            app_config.testing.geoip.cache.max_entries,
        )))
    } else {
        Ok(rate_limited)
    }
}
