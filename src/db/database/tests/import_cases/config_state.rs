use super::*;

#[tokio::test]
async fn config_activation_is_exclusive() {
    let db_path = test_database_path("xrat-config-state");
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
    let first_id = configs[0].id;
    let second_id = configs[1].id;

    db.set_active_config(first_id)
        .await
        .expect("activate first should succeed");
    db.set_active_config(second_id)
        .await
        .expect("activate second should succeed");

    let active = db
        .get_active_config()
        .await
        .expect("active query should succeed")
        .expect("active config should exist");
    let configs = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed");

    assert_eq!(active.id, second_id);
    assert_eq!(configs.iter().filter(|config| config.is_active).count(), 1);

    let _ = std::fs::remove_file(db_path);
}
