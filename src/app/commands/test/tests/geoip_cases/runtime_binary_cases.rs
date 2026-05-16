use std::path::PathBuf;

use super::super::super::*;

#[test]
fn resolves_xray_binary_from_runtime_paths() {
    let app_config = AppConfig::default();
    let runtime_paths = crate::app::context::RuntimePaths {
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

    let runtime_paths = crate::app::context::RuntimePaths {
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
