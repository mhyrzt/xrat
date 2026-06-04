use super::super::super::import_cases::test_node;
use super::*;

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

    let filter = ConfigListFilter {
        include_deleted: true,
        ..Default::default()
    };
    let configs = db.list_configs(&filter).await.expect("list should succeed");

    assert_eq!(configs.len(), 1);
    assert!(configs[0].is_deleted);

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

    let filter = ConfigListFilter {
        include_deleted: true,
        ..Default::default()
    };
    let configs = db.list_configs(&filter).await.expect("list should succeed");

    assert!(configs.is_empty());

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

    let filter = ConfigListFilter {
        only_deleted: true,
        ..Default::default()
    };
    let deleted_only = db.list_configs(&filter).await.expect("list should succeed");

    assert_eq!(deleted_only.len(), 1);
    assert!(deleted_only[0].is_deleted);

    let _ = std::fs::remove_file(db_path);
}
