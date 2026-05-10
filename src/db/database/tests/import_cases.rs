use super::{ConfigListFilter, Database, ImportSource, test_database_path};
use crate::db::model::SourceKind;
use crate::model::{Node, Protocol};

pub(super) fn test_node(name: &str) -> Node {
    Node {
        protocol: Protocol::Vless,
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: Some("uuid-123".to_string()),
        password: None,
        method: None,
        network: "ws".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("cdn.example.com".to_string()),
        host: Some("cdn.example.com".to_string()),
        path: Some("/socket".to_string()),
        name: Some(name.to_string()),
        extensions: None,
        raw_config: format!(
            "vless://uuid-123@example.com:443?type=ws&security=tls#{}",
            name
        ),
    }
}

#[tokio::test]
async fn imports_nodes_and_creates_subscription() {
    let db_path = test_database_path("xrat-import");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::Url,
        value: "https://example.com/sub".to_string(),
        name: Some("Example".to_string()),
    };

    let summary = db
        .import_nodes(&source, &[test_node("first")])
        .await
        .expect("import should succeed");

    assert_eq!(summary.imported_configs, 1);
    assert_eq!(summary.total_configs, 1);
    assert_eq!(db.get_subscription_count().await.expect("count"), 1);
    assert_eq!(db.get_config_count().await.expect("count"), 1);
    assert_eq!(db.get_connection_test_count().await.expect("count"), 0);
    assert_eq!(db.get_runtime_session_count().await.expect("count"), 0);

    let subscriptions = db.list_subscriptions().await.expect("subscriptions");
    assert_eq!(subscriptions.len(), 1);
    assert_eq!(subscriptions[0].source_kind, "url");
    assert_eq!(subscriptions[0].config_count, 1);

    let _ = std::fs::remove_file(db_path);
}

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

#[tokio::test]
async fn config_selection_and_activation_are_exclusive() {
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

    db.set_selected_config(first_id)
        .await
        .expect("select first should succeed");
    db.set_selected_config(second_id)
        .await
        .expect("select second should succeed");
    db.set_active_config(first_id)
        .await
        .expect("activate first should succeed");
    db.set_active_config(second_id)
        .await
        .expect("activate second should succeed");

    let selected = db
        .get_selected_config()
        .await
        .expect("selected query should succeed")
        .expect("selected config should exist");
    let active = db
        .get_active_config()
        .await
        .expect("active query should succeed")
        .expect("active config should exist");
    let configs = db
        .list_configs(&ConfigListFilter {
            ..ConfigListFilter::default()
        })
        .await
        .expect("list should succeed");

    assert_eq!(selected.id, second_id);
    assert_eq!(active.id, second_id);
    assert_eq!(
        configs.iter().filter(|config| config.is_selected).count(),
        1
    );
    assert_eq!(configs.iter().filter(|config| config.is_active).count(), 1);

    let _ = std::fs::remove_file(db_path);
}
