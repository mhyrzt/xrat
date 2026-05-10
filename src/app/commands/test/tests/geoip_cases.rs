use super::super::*;

#[test]
fn resolves_endpoint_meta_priority_with_real_mmdb_when_provided() {
    let Some(city_path) = std::env::var_os("XRAT_GEOIP_TEST_CITY_MMDB") else {
        return;
    };
    let Some(country_path) = std::env::var_os("XRAT_GEOIP_TEST_MMDB") else {
        return;
    };
    let Some(asn_path) = std::env::var_os("XRAT_GEOIP_TEST_ASN_MMDB") else {
        return;
    };

    let ip = "8.8.8.8";
    let meta = resolve_endpoint_meta(
        Some(ip),
        true,
        city_path.as_ref(),
        country_path.as_ref(),
        asn_path.as_ref(),
    );

    if let Some(city) = geoip::lookup_city_label(city_path.as_ref(), ip) {
        assert_eq!(meta.location.as_deref(), Some(city.as_str()));
        assert_eq!(
            meta.country.as_deref(),
            city.split('/').next().map(str::trim)
        );
        return;
    }

    if let Some(country) = geoip::lookup_country_iso(country_path.as_ref(), ip) {
        assert_eq!(meta.location.as_deref(), Some(country.as_str()));
        assert_eq!(meta.country.as_deref(), Some(country.as_str()));
        return;
    }

    if let Some(asn) = geoip::lookup_asn_label(asn_path.as_ref(), ip) {
        assert_eq!(meta.location.as_deref(), Some(asn.as_str()));
        assert!(meta.country.is_none());
        return;
    }

    panic!("expected at least one mmdb lookup to resolve for provided test assets");
}

#[test]
fn resolves_xray_binary_from_runtime_paths() {
    let app_config = AppConfig::default();
    let runtime_paths = crate::app::runtime::RuntimePaths {
        root_dir: "/tmp/xrat".into(),
        database_config: DatabaseConnectionConfig::Sqlite {
            path: "/tmp/xrat/db.sqlite".into(),
        },
        database_path: "/tmp/xrat/db.sqlite".into(),
        database_label: "/tmp/xrat/db.sqlite".to_string(),
        config_path: "/tmp/xrat/config.toml".into(),
        runtime_dir: "/tmp/xrat/runtime".into(),
        xray_path: "/tmp/xrat/bin/xray".into(),
        v2ray_path: "/tmp/xrat/bin/v2ray".into(),
        sing_box_path: "/tmp/xrat/bin/sing-box".into(),
    };

    let resolved = resolve_engine_binary_path(&app_config, &runtime_paths);

    assert_eq!(resolved, PathBuf::from("/tmp/xrat/bin/xray"));
}

#[test]
fn resolves_v2ray_binary_when_engine_is_v2ray() {
    let app_config = AppConfig {
        paths: crate::app::config::PathSettings {
            xray: Some("bin/xray".into()),
            v2ray: Some("/opt/v2ray/v2ray".into()),
            ..Default::default()
        },
        runtime: crate::app::config::RuntimeSettings {
            engine: "v2ray".to_string(),
            ..Default::default()
        },
        ..AppConfig::default()
    };

    let runtime_paths = crate::app::runtime::RuntimePaths {
        root_dir: "/tmp/xrat".into(),
        database_config: DatabaseConnectionConfig::Sqlite {
            path: "/tmp/xrat/db.sqlite".into(),
        },
        database_path: "/tmp/xrat/db.sqlite".into(),
        database_label: "/tmp/xrat/db.sqlite".to_string(),
        config_path: "/tmp/xrat/config.toml".into(),
        runtime_dir: "/tmp/xrat/runtime".into(),
        xray_path: "/tmp/xrat/bin/xray".into(),
        v2ray_path: "/opt/v2ray/v2ray".into(),
        sing_box_path: "/tmp/xrat/bin/sing-box".into(),
    };

    let resolved = resolve_engine_binary_path(&app_config, &runtime_paths);

    assert_eq!(resolved, PathBuf::from("/opt/v2ray/v2ray"));
}

pub(super) fn test_runtime_paths() -> crate::app::runtime::RuntimePaths {
    crate::app::runtime::RuntimePaths {
        root_dir: "/tmp/xrat".into(),
        database_config: DatabaseConnectionConfig::Sqlite {
            path: "/tmp/xrat/db.sqlite".into(),
        },
        database_path: "/tmp/xrat/db.sqlite".into(),
        database_label: "/tmp/xrat/db.sqlite".to_string(),
        config_path: "/tmp/xrat/config.toml".into(),
        runtime_dir: "/tmp/xrat/runtime".into(),
        xray_path: "xray".into(),
        v2ray_path: "v2ray".into(),
        sing_box_path: "sing-box".into(),
    }
}

pub(super) fn test_args(id: Option<i64>) -> TestArgs {
    TestArgs {
        id,
        enabled_only: false,
        active_only: false,
        selected_only: false,
        subscription: None,
        skip_icmp: false,
        skip_tcp: false,
        skip_real_delay: false,
        skip_download: false,
        skip_upload: false,
        test_url: None,
        download_url: None,
        upload_url: None,
        icmp_timeout_ms: None,
        tcp_timeout_ms: None,
        real_delay_timeout_ms: None,
        download_timeout_ms: None,
        upload_timeout_ms: None,
        concurrency: None,
        format: crate::cli::TestFormat::Tsv,
        output: None,
        sort_by: crate::cli::TestSortBy::Status,
        no_progress: false,
        ping: false,
        ping_interval_ms: 1000,
        latest_run_summary: false,
        country: None,
        asn: None,
    }
}
