use super::super::import_cases::test_node;
use super::super::*;

#[tokio::test]
async fn list_with_latest_tests_returns_config_and_test() {
    let db_path = test_database_path("xrat-server-ops-joined");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    db.import_nodes(&source, &[test_node("node-a")])
        .await
        .expect("import should succeed");

    let config = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed")
        .into_iter()
        .next()
        .expect("config should exist");

    db.insert_connection_test(&ConnectionTestInsert {
        run_id: None,
        config_id: config.id,
        icmp_ok: None,
        icmp_ms: None,
        tcp_ok: Some(true),
        tcp_ms: Some(50),
        real_delay_ok: Some(true),
        real_delay_ms: Some(200),
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

    let rows = db
        .list_configs_with_latest_tests(&ConfigListFilter::default())
        .await
        .expect("joined list should succeed");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].config.id, config.id);
    assert_eq!(rows[0].tcp_ok, Some(true));
    assert_eq!(rows[0].real_delay_ms, Some(200));
    assert!(rows[0].tested_at.is_some());

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn list_with_latest_tests_returns_null_test_when_no_test_exists() {
    let db_path = test_database_path("xrat-server-ops-no-test");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    db.import_nodes(&source, &[test_node("node-b")])
        .await
        .expect("import should succeed");

    let rows = db
        .list_configs_with_latest_tests(&ConfigListFilter::default())
        .await
        .expect("joined list should succeed");

    assert_eq!(rows.len(), 1);
    assert!(rows[0].test_id.is_none());
    assert!(rows[0].real_delay_ms.is_none());
    assert!(rows[0].tested_at.is_none());

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn top_by_real_delay_excludes_configs_without_delay_and_sorts_ascending() {
    let db_path = test_database_path("xrat-server-ops-top");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    db.import_nodes(
        &source,
        &[
            test_node("slow"),
            node_with_protocol("trojan", "fast"),
            node_with_protocol("trojan", "medium"),
        ],
    )
    .await
    .expect("import should succeed");

    let configs = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed");

    for (config, delay) in configs.iter().zip([500i64, 100, 300]) {
        db.insert_connection_test(&ConnectionTestInsert {
            run_id: None,
            config_id: config.id,
            icmp_ok: None,
            icmp_ms: None,
            tcp_ok: Some(true),
            tcp_ms: None,
            real_delay_ok: Some(true),
            real_delay_ms: if config.name.as_deref() == Some("slow") {
                None
            } else {
                Some(delay)
            },
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

    let top = db
        .list_top_configs_by_real_delay(10, &ConfigListFilter::default())
        .await
        .expect("top query should succeed");

    assert_eq!(top.len(), 2);
    assert_eq!(top[0].real_delay_ms, Some(100));
    assert_eq!(top[1].real_delay_ms, Some(300));

    let top_one = db
        .list_top_configs_by_real_delay(1, &ConfigListFilter::default())
        .await
        .expect("top-1 query should succeed");
    assert_eq!(top_one.len(), 1);
    assert_eq!(top_one[0].real_delay_ms, Some(100));

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn paginated_list_returns_correct_page_and_total() {
    let db_path = test_database_path("xrat-server-ops-paged");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    let nodes: Vec<_> = (0..5).map(|i| unique_node(&format!("node-{i}"))).collect();
    db.import_nodes(&source, &nodes)
        .await
        .expect("import should succeed");

    let filter = ConfigListFilter::default();
    let total = db
        .count_filtered_configs(&filter)
        .await
        .expect("count should succeed");
    assert_eq!(total, 5);

    let page_one = db
        .list_configs_paginated_with_latest_tests(&filter, 0, 2)
        .await
        .expect("page 1 should succeed");
    assert_eq!(page_one.len(), 2);

    let page_two = db
        .list_configs_paginated_with_latest_tests(&filter, 2, 2)
        .await
        .expect("page 2 should succeed");
    assert_eq!(page_two.len(), 2);

    let page_three = db
        .list_configs_paginated_with_latest_tests(&filter, 4, 2)
        .await
        .expect("page 3 should succeed");
    assert_eq!(page_three.len(), 1);

    assert_ne!(page_one[0].config.id, page_two[0].config.id);
    assert_ne!(page_two[0].config.id, page_three[0].config.id);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn protocol_filter_restricts_results() {
    let db_path = test_database_path("xrat-server-ops-protocol");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    db.import_nodes(
        &source,
        &[
            test_node("vless-node"),
            node_with_protocol("trojan", "trojan-node"),
        ],
    )
    .await
    .expect("import should succeed");

    let vless_filter = ConfigListFilter {
        protocol: Some("vless".to_string()),
        ..Default::default()
    };
    let vless_rows = db
        .list_configs_with_latest_tests(&vless_filter)
        .await
        .expect("vless filter should succeed");
    assert_eq!(vless_rows.len(), 1);
    assert_eq!(vless_rows[0].config.protocol, "vless");

    let trojan_filter = ConfigListFilter {
        protocol: Some("trojan".to_string()),
        ..Default::default()
    };
    let trojan_rows = db
        .list_configs_with_latest_tests(&trojan_filter)
        .await
        .expect("trojan filter should succeed");
    assert_eq!(trojan_rows.len(), 1);
    assert_eq!(trojan_rows[0].config.protocol, "trojan");

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn get_config_with_latest_test_returns_joined_record() {
    let db_path = test_database_path("xrat-server-ops-detail");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    db.import_nodes(&source, &[test_node("detail-node")])
        .await
        .expect("import should succeed");

    let config = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed")
        .into_iter()
        .next()
        .expect("config should exist");

    db.insert_connection_test(&ConnectionTestInsert {
        run_id: None,
        config_id: config.id,
        icmp_ok: None,
        icmp_ms: None,
        tcp_ok: Some(true),
        tcp_ms: Some(30),
        real_delay_ok: Some(true),
        real_delay_ms: Some(150),
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

    let row = db
        .get_config_with_latest_test(config.id)
        .await
        .expect("detail should succeed")
        .expect("detail should exist");
    assert_eq!(row.config.id, config.id);
    assert_eq!(row.tcp_ok, Some(true));
    assert_eq!(row.real_delay_ms, Some(150));

    let missing = db
        .get_config_with_latest_test(-1)
        .await
        .expect("missing should succeed");
    assert!(missing.is_none());

    let _ = std::fs::remove_file(db_path);
}

fn unique_node(name: &str) -> crate::model::Node {
    use crate::model::{Node, Protocol};
    Node {
        protocol: Protocol::Vless,
        address: format!("{name}.example.com"),
        port: 443,
        username: None,
        uuid: Some(format!("uuid-{name}")),
        password: None,
        method: None,
        network: "ws".to_string(),
        tls: Some("tls".to_string()),
        sni: None,
        host: None,
        path: None,
        name: Some(name.to_string()),
        extensions: None,
        raw_config: format!(
            "vless://uuid-{name}@{name}.example.com:443?type=ws&security=tls#{name}"
        ),
    }
}

fn node_with_protocol(protocol: &str, name: &str) -> crate::model::Node {
    use crate::model::{Node, Protocol};
    let proto = match protocol {
        "trojan" => Protocol::Trojan,
        "vless" => Protocol::Vless,
        _ => Protocol::Vless,
    };
    Node {
        protocol: proto,
        address: format!("{name}.example.com"),
        port: 443,
        username: None,
        uuid: Some(format!("uuid-{name}")),
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: Some("tls".to_string()),
        sni: None,
        host: None,
        path: None,
        name: Some(name.to_string()),
        extensions: None,
        raw_config: format!("{protocol}://uuid-{name}@{name}.example.com:443#{name}"),
    }
}
