use super::super::*;
use crate::app::config::AppConfig;
use crate::app::context::RuntimePaths;
use crate::db::{Database, DatabaseConnectionConfig, ImportSource, SourceKind};
use crate::model::{Node, Protocol};
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Hands each test_context its own inbound port block so replace tests that
/// launch the fake runtime can run in parallel without colliding on the fixed
/// default ports. Base 40000 keeps this range disjoint from the supervisor
/// test helper's range.
fn unique_port_base() -> u16 {
    static COUNTER: AtomicU16 = AtomicU16::new(0);
    let slot = COUNTER.fetch_add(1, Ordering::Relaxed) % 4000;
    40000 + slot * 4
}

pub(super) async fn test_context() -> AppContext {
    let root = std::env::temp_dir().join(format!(
        "xrat-runtime-service-{}",
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

    let mut app_config = AppConfig::default();
    let port_base = unique_port_base();
    app_config.runtime.socks.port = port_base;
    app_config.runtime.http.port = port_base + 1;
    app_config.runtime.shadowsocks.port = port_base + 2;

    AppContext {
        db,
        app_config,
        runtime_paths: RuntimePaths {
            root_dir: root.clone(),
            database_config,
            database_path: root.join("db.sqlite"),
            database_label: root.join("db.sqlite").display().to_string(),
            config_path: root.join("config.toml"),
            runtime_dir: root.join("runtime"),
            xray_path: "xray".into(),
            v2ray_path: "v2ray".into(),
            sing_box_path: "sing-box".into(),
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

pub(super) fn test_node() -> Node {
    test_node_with("example.com", "test")
}

pub(super) fn test_node_with(address: &str, name: &str) -> Node {
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
