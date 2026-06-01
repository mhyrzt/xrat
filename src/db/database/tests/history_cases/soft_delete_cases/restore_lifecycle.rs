use super::super::super::import_cases::test_node;
use super::*;

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
