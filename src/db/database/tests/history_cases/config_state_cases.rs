use super::super::import_cases::test_node;
use super::super::*;

#[tokio::test]
async fn disabling_configs_clears_activation() {
    let db_path = test_database_path("xrat-config-visibility");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    db.import_nodes(&source, &[test_node("first")])
        .await
        .expect("import should succeed");

    let config = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed")
        .into_iter()
        .next()
        .expect("config should exist");
    db.set_active_config(config.id)
        .await
        .expect("activate should succeed");
    db.set_config_enabled(config.id, false)
        .await
        .expect("disable should succeed");

    let disabled = db
        .get_config_by_id(config.id)
        .await
        .expect("query should succeed")
        .expect("config should still exist");
    assert!(!disabled.is_enabled);
    assert!(!disabled.is_active);

    let _ = std::fs::remove_file(db_path);
}
