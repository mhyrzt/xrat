use super::*;
use crate::db::RefMatch;

#[tokio::test]
async fn configs_get_unique_refs_and_resolve_by_prefix() {
    let db_path = test_database_path("xrat-config-refs");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };
    let first = test_node("first");
    let mut second = test_node("second");
    second.address = "second.example.com".to_string();

    db.import_nodes(&source, &[first, second])
        .await
        .expect("import should succeed");

    let configs = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed");
    assert_eq!(configs.len(), 2);
    assert_eq!(configs[0].r#ref.len(), 12);
    assert_ne!(configs[0].r#ref, configs[1].r#ref);

    let full = configs[0].r#ref.clone();
    let resolved = db
        .resolve_config_ref_prefix(&full)
        .await
        .expect("resolve should succeed");
    assert_eq!(resolved, RefMatch::Unique(configs[0].id));

    let missing = db
        .resolve_config_ref_prefix("ffffffffffff")
        .await
        .expect("resolve should succeed");
    assert_eq!(missing, RefMatch::None);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn ambiguous_prefix_is_detected() {
    let db_path = test_database_path("xrat-config-refs-ambiguous");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };
    let mut nodes = Vec::new();
    for index in 0..5 {
        let mut node = test_node(&format!("node-{index}"));
        node.address = format!("node-{index}.example.com");
        nodes.push(node);
    }
    db.import_nodes(&source, &nodes)
        .await
        .expect("import should succeed");

    // The empty prefix matches every config, so resolution is ambiguous.
    let resolved = db
        .resolve_config_ref_prefix("")
        .await
        .expect("resolve should succeed");
    assert_eq!(resolved, RefMatch::Ambiguous);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn subscription_refs_resolve_by_prefix() {
    let db_path = test_database_path("xrat-subscription-refs");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::Url,
        value: "https://example.com/sub".to_string(),
        name: Some("sub".to_string()),
    };
    db.import_nodes(&source, &[test_node("first")])
        .await
        .expect("import should succeed");

    let subscriptions = db.list_subscriptions().await.expect("list should succeed");
    assert_eq!(subscriptions.len(), 1);
    assert_eq!(subscriptions[0].r#ref.len(), 12);

    let resolved = db
        .resolve_subscription_ref_prefix(&subscriptions[0].r#ref)
        .await
        .expect("resolve should succeed");
    assert_eq!(resolved, RefMatch::Unique(subscriptions[0].id));

    let _ = std::fs::remove_file(db_path);
}
