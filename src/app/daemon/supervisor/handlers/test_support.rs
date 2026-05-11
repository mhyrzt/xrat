use crate::app::config::AppConfig;
use crate::app::runtime::{AppContext, RuntimePaths};
use crate::db::{Database, DatabaseConnectionConfig, ImportSource, SourceKind};
use crate::model::{Node, Protocol};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) async fn test_context(prefix: &str) -> AppContext {
    let root = std::env::temp_dir().join(format!(
        "xrat-supervisor-{prefix}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).expect("root should be created");
    let database_config = DatabaseConnectionConfig::Sqlite {
        path: root.join("db.sqlite"),
    };
    let db = Database::connect(&database_config)
        .await
        .expect("database should connect");
    AppContext {
        db,
        app_config: AppConfig::default(),
        runtime_paths: RuntimePaths {
            root_dir: root.clone(),
            database_config,
            database_path: root.join("db.sqlite"),
            database_label: root.join("db.sqlite").display().to_string(),
            config_path: root.join("config.toml"),
            runtime_dir: root.join("runtime"),
            xray_path: PathBuf::from("xray"),
            v2ray_path: PathBuf::from("v2ray"),
            sing_box_path: PathBuf::from("sing-box"),
        },
    }
}

pub(super) fn test_source() -> ImportSource {
    ImportSource {
        kind: SourceKind::RawText,
        value: "test".to_string(),
        name: Some("test".to_string()),
    }
}

pub(super) fn test_node(address: &str, name: &str) -> Node {
    Node {
        protocol: Protocol::Vless,
        address: address.to_string(),
        port: 443,
        username: None,
        uuid: Some("00000000-0000-0000-0000-000000000000".to_string()),
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: Some("tls".to_string()),
        sni: Some(address.to_string()),
        host: None,
        path: None,
        name: Some(name.to_string()),
        extensions: None,
        raw_config: format!(
            "vless://00000000-0000-0000-0000-000000000000@{address}:443?security=tls#{name}"
        ),
    }
}
