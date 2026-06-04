use super::super::super::import_cases::test_node;
use super::*;

#[tokio::test]
async fn purge_removes_only_soft_deleted_configs() {
    let db_path = test_database_path("xrat-purge-deleted");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    let mut keep = test_node("keep");
    keep.address = "keep.example.com".to_string();
    let mut drop = test_node("drop");
    drop.address = "drop.example.com".to_string();
    db.import_nodes(&source, &[keep, drop])
        .await
        .expect("import should succeed");

    let configs = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed");
    assert_eq!(configs.len(), 2);
    let drop_id = configs[1].id;

    // A dependent runtime_sessions row must not block the hard delete.
    db.insert_runtime_session(&RuntimeSessionInsert {
        config_id: Some(drop_id),
        status: RuntimeSessionStatus::Stopped,
        socks_host: None,
        socks_port: None,
        http_host: None,
        http_port: None,
        shadowsocks_host: None,
        shadowsocks_port: None,
        process_id: None,
        failure_reason: None,
        started_at: Some("2025-01-01T10:00:00Z".to_string()),
        stopped_at: Some("2025-01-01T10:05:00Z".to_string()),
    })
    .await
    .expect("session insert should succeed");

    db.delete_config(drop_id)
        .await
        .expect("soft delete should succeed");

    assert_eq!(
        db.count_deleted_configs()
            .await
            .expect("count should succeed"),
        1
    );

    let purged = db
        .purge_deleted_configs()
        .await
        .expect("purge should succeed");
    assert_eq!(purged, 1);

    assert_eq!(
        db.count_deleted_configs()
            .await
            .expect("count should succeed"),
        0
    );
    assert!(
        db.get_config_by_id(drop_id)
            .await
            .expect("query should succeed")
            .is_none()
    );

    let remaining = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed");
    assert_eq!(remaining.len(), 1);

    // The dependent session row was removed alongside the config.
    assert_eq!(
        db.get_runtime_session_count()
            .await
            .expect("count should succeed"),
        0
    );

    let _ = std::fs::remove_file(db_path);
}
