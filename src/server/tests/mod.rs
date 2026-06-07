use crate::db::{
    ConnectionTestInsert, Database, DatabaseConnectionConfig, ImportSource, SourceKind,
};
use crate::model::{Node, Protocol};
use crate::server::ServerState;

pub(super) mod bind_addr;
pub(super) mod routes_b64;
pub(super) mod routes_configs;
pub(super) mod routes_health;
pub(super) mod routes_json;
pub(super) mod routes_pac;

pub(super) async fn multi_config_state(api_key: Option<&str>, count: usize) -> ServerState {
    let db = Database::connect(&DatabaseConnectionConfig::Sqlite {
        path: std::env::temp_dir().join(format!(
            "xrat-server-multi-{}.sqlite",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        )),
    })
    .await
    .expect("database should connect");

    let nodes: Vec<Node> = (0..count)
        .map(|i| Node {
            protocol: Protocol::Vless,
            address: format!("node-{i}.example.com"),
            port: 443,
            username: None,
            uuid: Some(format!("00000000-0000-0000-0000-{i:012}")),
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: Some("tls".to_string()),
            sni: None,
            host: None,
            path: None,
            name: Some(format!("node-{i}")),
            extensions: None,
            raw_config: format!(
                "vless://00000000-0000-0000-0000-{i:012}@node-{i}.example.com:443#node-{i}"
            ),
        })
        .collect();

    db.import_nodes(
        &ImportSource {
            kind: SourceKind::RawText,
            value: "test".to_string(),
            name: None,
        },
        &nodes,
    )
    .await
    .expect("nodes should import");

    let configs = db
        .list_configs(&Default::default())
        .await
        .expect("configs should load");

    for (i, config) in configs.iter().enumerate() {
        db.insert_connection_test(&ConnectionTestInsert {
            run_id: None,
            config_id: config.id,
            icmp_ok: None,
            icmp_ms: None,
            tcp_ok: Some(true),
            tcp_ms: Some(40 + i as i64),
            real_delay_ok: Some(true),
            real_delay_ms: Some((count as i64 - i as i64) * 100),
            download_mbps: None,
            upload_mbps: None,
            connect_ms: None,
            ttfb_ms: None,
            http_status: None,
            endpoint_ip: None,
            endpoint_location: None,
            endpoint_country: None,
            endpoint_asn: None,
            failure_kind: None,
            failure_reason: None,
        })
        .await
        .expect("test should insert");
    }

    ServerState {
        db,
        api_key: api_key.map(str::to_string),
        pac_enabled: true,
        pac_allowed_hosts: crate::app::config::defaults::DEFAULT_SERVER_PAC_ALLOWED_HOSTS
            .iter()
            .map(|host| host.to_string())
            .collect(),
        pac_rules: crate::server::PacRules::default(),
    }
}

pub(super) async fn populated_state(api_key: Option<&str>) -> ServerState {
    let db = Database::connect(&DatabaseConnectionConfig::Sqlite {
        path: std::env::temp_dir().join(format!(
            "xrat-server-route-{}.sqlite",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        )),
    })
    .await
    .expect("database should connect");

    db.import_nodes(
        &ImportSource {
            kind: SourceKind::RawText,
            value: "test".to_string(),
            name: None,
        },
        &[test_node()],
    )
    .await
    .expect("node should import");
    let config = db
        .list_configs(&Default::default())
        .await
        .expect("configs should load")
        .into_iter()
        .next()
        .expect("config should exist");
    db.insert_connection_test(&ConnectionTestInsert {
        run_id: None,
        config_id: config.id,
        icmp_ok: None,
        icmp_ms: None,
        tcp_ok: Some(true),
        tcp_ms: Some(45),
        real_delay_ok: Some(true),
        real_delay_ms: Some(123),
        download_mbps: None,
        upload_mbps: None,
        connect_ms: None,
        ttfb_ms: None,
        http_status: None,
        endpoint_ip: None,
        endpoint_location: None,
        endpoint_country: None,
        endpoint_asn: None,
        failure_kind: None,
        failure_reason: None,
    })
    .await
    .expect("test should insert");

    ServerState {
        db,
        api_key: api_key.map(str::to_string),
        pac_enabled: true,
        pac_allowed_hosts: crate::app::config::defaults::DEFAULT_SERVER_PAC_ALLOWED_HOSTS
            .iter()
            .map(|host| host.to_string())
            .collect(),
        pac_rules: crate::server::PacRules::default(),
    }
}

pub(super) fn test_node() -> Node {
    Node {
        protocol: Protocol::Vless,
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: Some("00000000-0000-0000-0000-000000000001".to_string()),
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("example.com".to_string()),
        host: None,
        path: None,
        name: Some("test-node".to_string()),
        extensions: None,
        raw_config:
            "vless://00000000-0000-0000-0000-000000000001@example.com:443?security=tls#test-node"
                .to_string(),
    }
}
