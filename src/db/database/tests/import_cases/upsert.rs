use super::*;

#[tokio::test]
async fn upsert_updates_existing_config_without_creating_duplicates() {
    let db_path = test_database_path("xrat-upsert");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };
    let node = test_node("first");

    db.import_nodes(&source, std::slice::from_ref(&node))
        .await
        .expect("first import should succeed");
    db.import_nodes(&source, &[test_node("updated")])
        .await
        .expect("second import should succeed");

    assert_eq!(db.get_config_count().await.expect("count"), 1);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn import_round_trip_preserves_json_extensions_without_raw_link_fallback() {
    let db_path = test_database_path("xrat-extension-round-trip");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::RawText,
        value: "test".to_string(),
        name: None,
    };
    let mut node = test_node("extensions");
    node.raw_config = "not-a-parseable-link".to_string();
    node.extensions = Some(std::collections::BTreeMap::from([
        ("multiMode".to_string(), serde_json::json!(true)),
        ("header".to_string(), serde_json::json!(["first", "second"])),
    ]));

    db.import_nodes(&source, &[node.clone()])
        .await
        .expect("import should succeed");
    let record = db
        .list_configs(&Default::default())
        .await
        .expect("configs should load")
        .into_iter()
        .next()
        .expect("config should exist");
    assert!(record.extensions_json.is_some());
    assert_eq!(
        crate::db::node_from_record(&record)
            .expect("record should reconstruct")
            .extensions,
        node.extensions
    );

    let _ = std::fs::remove_file(db_path);
}
