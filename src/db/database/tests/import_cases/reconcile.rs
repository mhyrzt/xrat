use super::*;

fn node_at(name: &str, address: &str) -> Node {
    let mut node = test_node(name);
    node.address = address.to_string();
    node
}

#[tokio::test]
async fn refresh_soft_deletes_provider_removed_configs() {
    let db_path = test_database_path("xrat-reconcile");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::Url,
        value: "https://example.com/sub".to_string(),
        name: None,
    };

    let initial = db
        .import_nodes(
            &source,
            &[node_at("a", "a.example.com"), node_at("b", "b.example.com")],
        )
        .await
        .expect("initial import should succeed");
    assert_eq!(initial.imported_configs, 2);
    assert_eq!(initial.removed_configs, 0);
    assert_eq!(db.get_config_count().await.expect("count"), 2);

    // Refresh where the provider dropped "b".
    let refreshed = db
        .import_nodes(&source, &[node_at("a", "a.example.com")])
        .await
        .expect("refresh should succeed");
    assert_eq!(refreshed.imported_configs, 1);
    assert_eq!(refreshed.removed_configs, 1);
    assert_eq!(db.get_config_count().await.expect("active count"), 1);

    let active = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list active");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].address, "a.example.com");

    let all = db
        .list_configs(&ConfigListFilter {
            include_deleted: true,
            ..Default::default()
        })
        .await
        .expect("list all");
    assert_eq!(all.len(), 2, "removed config is soft-deleted, not purged");

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn refresh_restores_returning_config() {
    let db_path = test_database_path("xrat-reconcile-restore");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::Url,
        value: "https://example.com/restore".to_string(),
        name: None,
    };

    db.import_nodes(
        &source,
        &[node_at("a", "a.example.com"), node_at("b", "b.example.com")],
    )
    .await
    .expect("initial import");
    db.import_nodes(&source, &[node_at("a", "a.example.com")])
        .await
        .expect("refresh dropping b");

    // Provider brings "b" back: the upsert clears its soft-delete flag.
    let restored = db
        .import_nodes(
            &source,
            &[node_at("a", "a.example.com"), node_at("b", "b.example.com")],
        )
        .await
        .expect("refresh restoring b");
    assert_eq!(restored.removed_configs, 0);
    assert_eq!(db.get_config_count().await.expect("active count"), 2);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn empty_refresh_removes_nothing() {
    let db_path = test_database_path("xrat-reconcile-empty");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::Url,
        value: "https://example.com/empty".to_string(),
        name: None,
    };

    db.import_nodes(&source, &[node_at("a", "a.example.com")])
        .await
        .expect("initial import");
    let summary = db
        .import_nodes(&source, &[])
        .await
        .expect("empty refresh should succeed");
    assert_eq!(summary.removed_configs, 0);
    assert_eq!(db.get_config_count().await.expect("count"), 1);

    let _ = std::fs::remove_file(db_path);
}
