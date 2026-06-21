use super::super::super::import_cases::test_node;
use super::*;

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
            dial_endpoint_ip: None,
            dial_endpoint_location: None,
            dial_endpoint_country: None,
            dial_endpoint_asn: None,
            dial_endpoint_geoip_source: None,
            dial_endpoint_fronting: None,
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
