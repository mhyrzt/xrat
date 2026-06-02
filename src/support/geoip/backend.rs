use std::sync::Arc;

use crate::app::AppError;
use crate::app::config::{AppConfig, GeoIpBackend};
use crate::app::context::RuntimePaths;
use crate::app::paths::mmdb;

use super::{GeoIpLookup, LocalMmdbLookup, RemoteIpWhoisLookup};

pub fn build_lookup_chain(
    app_config: &AppConfig,
    runtime_paths: &RuntimePaths,
) -> crate::app::Result<Arc<dyn GeoIpLookup>> {
    validate_geoip_settings(&app_config.testing.geoip)?;

    match app_config.testing.geoip.backend {
        GeoIpBackend::Mmdb => Ok(Arc::new(LocalMmdbLookup::new(
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
        ))),
        GeoIpBackend::IpWhois => Ok(Arc::new(RemoteIpWhoisLookup::new(
            app_config.testing.geoip.remote.endpoint.clone(),
            std::time::Duration::from_millis(app_config.testing.geoip.remote.timeout_ms),
        )?)),
        GeoIpBackend::IpApi | GeoIpBackend::Chain => Err(AppError::InvalidArgument(
            "this geoip backend is not implemented yet".to_string(),
        )),
        GeoIpBackend::None => Err(AppError::InvalidArgument(
            "[testing.geoip].backend cannot be 'none'".to_string(),
        )),
    }
}

fn validate_geoip_settings(
    settings: &crate::app::config::GeoIpTestSettings,
) -> crate::app::Result<()> {
    if settings.backend == GeoIpBackend::None {
        return Err(AppError::InvalidArgument(
            "[testing.geoip].backend cannot be 'none'".to_string(),
        ));
    }

    if settings.backend != GeoIpBackend::Chain && settings.fallback != GeoIpBackend::None {
        return Err(AppError::InvalidArgument(
            "[testing.geoip].fallback requires backend = 'chain'".to_string(),
        ));
    }

    if settings.backend == GeoIpBackend::Chain && settings.fallback == GeoIpBackend::None {
        return Err(AppError::InvalidArgument(
            "[testing.geoip].fallback is required when backend = 'chain'".to_string(),
        ));
    }

    if settings.backend == settings.fallback && settings.fallback != GeoIpBackend::None {
        return Err(AppError::InvalidArgument(
            "[testing.geoip].fallback must differ from backend".to_string(),
        ));
    }

    if settings.cache.enabled && (settings.cache.ttl_secs == 0 || settings.cache.max_entries == 0) {
        return Err(AppError::InvalidArgument(
            "[testing.geoip.cache] ttl_secs and max_entries must be positive when cache is enabled"
                .to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::config::{GeoIpCacheSettings, GeoIpTestSettings};
    use crate::db::DatabaseConnectionConfig;

    fn runtime_paths() -> RuntimePaths {
        RuntimePaths {
            root_dir: "/tmp/xrat-root".into(),
            database_config: DatabaseConnectionConfig::Sqlite {
                path: "/tmp/xrat-root/db.sqlite".into(),
            },
            database_path: "/tmp/xrat-root/db.sqlite".into(),
            database_label: "/tmp/xrat-root/db.sqlite".to_string(),
            config_path: "/tmp/xrat-root/config.toml".into(),
            runtime_dir: "/tmp/xrat-root/runtime".into(),
            xray_path: "xray".into(),
            v2ray_path: "v2ray".into(),
            sing_box_path: "sing-box".into(),
        }
    }

    #[test]
    fn builds_mmdb_lookup_chain() {
        let app_config = AppConfig::default();
        let lookup =
            build_lookup_chain(&app_config, &runtime_paths()).expect("lookup should build");

        assert_eq!(lookup.backend_name(), "mmdb");
    }

    #[test]
    fn builds_ipwhois_lookup_chain() {
        let app_config = AppConfig {
            testing: crate::app::config::TestingSettings {
                geoip: crate::app::config::GeoIpTestSettings {
                    backend: GeoIpBackend::IpWhois,
                    ..crate::app::config::GeoIpTestSettings::default()
                },
                ..crate::app::config::TestingSettings::default()
            },
            ..AppConfig::default()
        };

        let lookup =
            build_lookup_chain(&app_config, &runtime_paths()).expect("lookup should build");

        assert_eq!(lookup.backend_name(), "ipwhois");
    }

    #[test]
    fn rejects_same_fallback_as_backend() {
        let settings = GeoIpTestSettings {
            backend: GeoIpBackend::Chain,
            fallback: GeoIpBackend::Chain,
            ..GeoIpTestSettings::default()
        };

        let error = validate_geoip_settings(&settings).expect_err("settings should fail");
        assert!(error.to_string().contains("fallback must differ"));
    }

    #[test]
    fn rejects_zero_cache_values_when_enabled() {
        let settings = GeoIpTestSettings {
            cache: GeoIpCacheSettings {
                enabled: true,
                ttl_secs: 0,
                max_entries: 0,
            },
            ..GeoIpTestSettings::default()
        };

        let error = validate_geoip_settings(&settings).expect_err("settings should fail");
        assert!(error.to_string().contains("must be positive"));
    }

    #[test]
    fn rejects_missing_fallback_for_chain_backend() {
        let settings = GeoIpTestSettings {
            backend: GeoIpBackend::Chain,
            fallback: GeoIpBackend::None,
            ..GeoIpTestSettings::default()
        };

        let error = validate_geoip_settings(&settings).expect_err("settings should fail");
        assert!(error.to_string().contains("fallback is required"));
    }
}
