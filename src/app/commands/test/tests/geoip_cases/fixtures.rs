use super::super::super::*;

pub(crate) fn test_runtime_paths() -> crate::app::context::RuntimePaths {
    crate::app::context::RuntimePaths {
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

pub(crate) fn test_args(id: Option<i64>) -> TestArgs {
    TestArgs {
        id,
        enabled_only: false,
        active_only: false,
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
