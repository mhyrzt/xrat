use super::super::super::import_cases::test_node;
use super::*;

async fn seed_three(db: &Database) -> Vec<i64> {
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };
    let mut nodes = Vec::new();
    for index in 0..3 {
        let mut node = test_node(&format!("node-{index}"));
        node.address = format!("host{index}.example.com");
        node.uuid = Some(format!("uuid-{index}"));
        nodes.push(node);
    }
    db.import_nodes(&source, &nodes)
        .await
        .expect("import should succeed");

    let filter = ConfigListFilter {
        include_deleted: true,
        ..Default::default()
    };
    let mut ids: Vec<i64> = db
        .list_configs(&filter)
        .await
        .expect("list should succeed")
        .into_iter()
        .map(|config| config.id)
        .collect();
    ids.sort_unstable();
    ids
}

#[tokio::test]
async fn delete_configs_soft_deletes_only_targeted_ids() {
    let db_path = test_database_path("xrat-bulk-soft-delete");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let ids = seed_three(&db).await;

    let affected = db
        .delete_configs(&[ids[0], ids[1]])
        .await
        .expect("bulk delete should succeed");
    assert_eq!(affected, 2);

    let filter = ConfigListFilter {
        include_deleted: true,
        ..Default::default()
    };
    let configs = db.list_configs(&filter).await.expect("list should succeed");
    for config in configs {
        if config.id == ids[2] {
            assert!(!config.is_deleted);
        } else {
            assert!(config.is_deleted);
            assert!(!config.is_active);
        }
    }

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn restore_configs_clears_deleted_flag_for_targeted_ids() {
    let db_path = test_database_path("xrat-bulk-restore");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let ids = seed_three(&db).await;

    db.delete_configs(&ids)
        .await
        .expect("bulk delete should succeed");
    let affected = db
        .restore_configs(&[ids[0]])
        .await
        .expect("bulk restore should succeed");
    assert_eq!(affected, 1);

    let restored = db
        .get_config_by_id(ids[0])
        .await
        .expect("query should succeed")
        .expect("config should exist");
    assert!(!restored.is_deleted);
    assert!(restored.deleted_at.is_none());

    let still_deleted = db
        .get_config_by_id(ids[1])
        .await
        .expect("query should succeed")
        .expect("config should exist");
    assert!(still_deleted.is_deleted);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn hard_delete_configs_removes_targeted_rows() {
    let db_path = test_database_path("xrat-bulk-hard-delete");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let ids = seed_three(&db).await;

    let affected = db
        .hard_delete_configs(&[ids[0], ids[1]])
        .await
        .expect("bulk hard delete should succeed");
    assert_eq!(affected, 2);

    assert!(
        db.get_config_by_id(ids[0])
            .await
            .expect("query should succeed")
            .is_none()
    );
    assert!(
        db.get_config_by_id(ids[2])
            .await
            .expect("query should succeed")
            .is_some()
    );

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn bulk_mutations_noop_on_empty_ids() {
    let db_path = test_database_path("xrat-bulk-empty");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    seed_three(&db).await;

    assert_eq!(db.delete_configs(&[]).await.expect("ok"), 0);
    assert_eq!(db.restore_configs(&[]).await.expect("ok"), 0);
    assert_eq!(db.hard_delete_configs(&[]).await.expect("ok"), 0);

    let _ = std::fs::remove_file(db_path);
}
