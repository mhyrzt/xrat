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
