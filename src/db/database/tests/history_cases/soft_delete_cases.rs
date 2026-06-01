use super::super::import_cases::test_node;
use super::super::*;

#[tokio::test]
async fn soft_delete_sets_is_deleted_and_deleted_at() {
    let db_path = test_database_path("xrat-soft-delete");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    db.import_nodes(&source, &[test_node("test")])
        .await
        .expect("import should succeed");

    let config = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed")
        .into_iter()
        .next()
        .expect("config should exist");

    db.delete_config(config.id)
        .await
        .expect("delete should succeed");

    let deleted = db
        .get_config_by_id(config.id)
        .await
        .expect("query should succeed")
        .expect("config should still exist");

    assert!(deleted.is_deleted);
    assert!(deleted.deleted_at.is_some());
    assert!(!deleted.is_active);
    assert!(!deleted.is_selected);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn soft_delete_excludes_from_default_list() {
    let db_path = test_database_path("xrat-soft-delete-list");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    db.import_nodes(&source, &[test_node("test")])
        .await
        .expect("import should succeed");

    let config = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed")
        .into_iter()
        .next()
        .expect("config should exist");

    db.delete_config(config.id)
        .await
        .expect("delete should succeed");

    let configs = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed");

    assert!(configs.is_empty());

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn soft_delete_visible_with_include_deleted_filter() {
    let db_path = test_database_path("xrat-soft-delete-include");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    db.import_nodes(&source, &[test_node("test")])
        .await
        .expect("import should succeed");

    let config = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed")
        .into_iter()
        .next()
        .expect("config should exist");

    db.delete_config(config.id)
        .await
        .expect("delete should succeed");

    let mut filter = ConfigListFilter::default();
    filter.include_deleted = true;
    let configs = db.list_configs(&filter).await.expect("list should succeed");

    assert_eq!(configs.len(), 1);
    assert!(configs[0].is_deleted);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn restore_clears_is_deleted_and_deleted_at() {
    let db_path = test_database_path("xrat-soft-delete-restore");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    db.import_nodes(&source, &[test_node("test")])
        .await
        .expect("import should succeed");

    let config = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed")
        .into_iter()
        .next()
        .expect("config should exist");

    db.delete_config(config.id)
        .await
        .expect("delete should succeed");
    db.restore_config(config.id)
        .await
        .expect("restore should succeed");

    let restored = db
        .get_config_by_id(config.id)
        .await
        .expect("query should succeed")
        .expect("config should exist");

    assert!(!restored.is_deleted);
    assert!(restored.deleted_at.is_none());

    let configs = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed");

    assert_eq!(configs.len(), 1);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn hard_delete_removes_row_completely() {
    let db_path = test_database_path("xrat-hard-delete");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    db.import_nodes(&source, &[test_node("test")])
        .await
        .expect("import should succeed");

    let config = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed")
        .into_iter()
        .next()
        .expect("config should exist");

    db.hard_delete_config(config.id)
        .await
        .expect("hard delete should succeed");

    let result = db
        .get_config_by_id(config.id)
        .await
        .expect("query should succeed");

    assert!(result.is_none());

    let mut filter = ConfigListFilter::default();
    filter.include_deleted = true;
    let configs = db.list_configs(&filter).await.expect("list should succeed");

    assert!(configs.is_empty());

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn reimport_revives_soft_deleted_config() {
    let db_path = test_database_path("xrat-reimport-revive");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };
    let node = test_node("test");

    db.import_nodes(&source, &[node.clone()])
        .await
        .expect("import should succeed");

    let config = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed")
        .into_iter()
        .next()
        .expect("config should exist");

    db.delete_config(config.id)
        .await
        .expect("delete should succeed");

    let deleted = db
        .get_config_by_id(config.id)
        .await
        .expect("query should succeed")
        .expect("config should exist");
    assert!(deleted.is_deleted);

    db.import_nodes(&source, &[node])
        .await
        .expect("reimport should succeed");

    let revived = db
        .get_config_by_id(config.id)
        .await
        .expect("query should succeed")
        .expect("config should exist");

    assert!(!revived.is_deleted);
    assert!(revived.deleted_at.is_none());

    let configs = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed");

    assert_eq!(configs.len(), 1);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn deleted_only_filter_shows_only_deleted() {
    let db_path = test_database_path("xrat-deleted-only");
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
    second.uuid = Some("uuid-456".to_string());

    db.import_nodes(&source, &[first.clone(), second.clone()])
        .await
        .expect("import should succeed");

    let configs = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed");

    db.delete_config(configs[0].id)
        .await
        .expect("delete should succeed");

    let mut filter = ConfigListFilter::default();
    filter.only_deleted = true;
    let deleted_only = db.list_configs(&filter).await.expect("list should succeed");

    assert_eq!(deleted_only.len(), 1);
    assert!(deleted_only[0].is_deleted);

    let _ = std::fs::remove_file(db_path);
}
