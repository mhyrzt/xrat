use super::super::{
    CachedLookup, ChainedLookup, GeoIpLookup, LocalMmdbLookup, RateLimitedLookup,
    RemoteIpApiLookup, RemoteIpWhoisLookup,
};
use super::validation::validate_geoip_settings;
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
    let lookup = build_lookup_chain(&app_config, &runtime_paths()).expect("lookup should build");

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

    let lookup = build_lookup_chain(&app_config, &runtime_paths()).expect("lookup should build");

    assert_eq!(lookup.backend_name(), "ipwhois");
}

#[test]
fn builds_ip_api_lookup_chain() {
    let app_config = AppConfig {
        testing: crate::app::config::TestingSettings {
            geoip: crate::app::config::GeoIpTestSettings {
                backend: GeoIpBackend::IpApi,
                ..crate::app::config::GeoIpTestSettings::default()
            },
            ..crate::app::config::TestingSettings::default()
        },
        ..AppConfig::default()
    };

    let lookup = build_lookup_chain(&app_config, &runtime_paths()).expect("lookup should build");

    assert_eq!(lookup.backend_name(), "ip-api");
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

#[test]
fn rejects_mmdb_fallback_for_chain_backend() {
    let settings = GeoIpTestSettings {
        backend: GeoIpBackend::Chain,
        fallback: GeoIpBackend::Mmdb,
        ..GeoIpTestSettings::default()
    };

    let error = validate_geoip_settings(&settings).expect_err("settings should fail");
    assert!(error.to_string().contains("must be ipwhois or ip-api"));
}

#[test]
fn builds_chain_lookup() {
    let app_config = AppConfig {
        testing: crate::app::config::TestingSettings {
            geoip: crate::app::config::GeoIpTestSettings {
                backend: GeoIpBackend::Chain,
                fallback: GeoIpBackend::IpWhois,
                ..crate::app::config::GeoIpTestSettings::default()
            },
            ..crate::app::config::TestingSettings::default()
        },
        ..AppConfig::default()
    };

    let lookup = build_lookup_chain(&app_config, &runtime_paths()).expect("lookup should build");

    assert_eq!(lookup.backend_name(), "mmdb");
}
