use super::*;

#[tokio::test]
async fn imports_nodes_and_creates_subscription() {
    let db_path = test_database_path("xrat-import");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::Url,
        value: "https://example.com/sub".to_string(),
        name: Some("Example".to_string()),
    };

    let summary = db
        .import_nodes(&source, &[test_node("first")])
        .await
        .expect("import should succeed");

    assert_eq!(summary.imported_configs, 1);
    assert_eq!(summary.total_configs, 1);
    assert_eq!(db.get_subscription_count().await.expect("count"), 1);
    assert_eq!(db.get_config_count().await.expect("count"), 1);
    assert_eq!(db.get_connection_test_count().await.expect("count"), 0);
    assert_eq!(db.get_runtime_session_count().await.expect("count"), 0);

    let subscriptions = db.list_subscriptions().await.expect("subscriptions");
    assert_eq!(subscriptions.len(), 1);
    assert_eq!(subscriptions[0].source_kind, "url");
    assert_eq!(subscriptions[0].config_count, 1);

    let _ = std::fs::remove_file(db_path);
}
