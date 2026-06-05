use super::*;
use crate::support::time::now_epoch_seconds;

#[tokio::test]
async fn lists_only_due_url_subscriptions() {
    let db_path = test_database_path("xrat-refresh-due");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");

    let url_source = ImportSource {
        kind: SourceKind::Url,
        value: "https://example.com/due".to_string(),
        name: None,
    };
    db.import_nodes(&url_source, &[test_node("a")])
        .await
        .expect("url import should succeed");

    // Non-URL sources are never auto-refreshable.
    let file_source = ImportSource {
        kind: SourceKind::File,
        value: "local.txt".to_string(),
        name: None,
    };
    let mut file_node = test_node("b");
    file_node.address = "file-node.example.com".to_string();
    db.import_nodes(&file_source, &[file_node])
        .await
        .expect("file import should succeed");

    let now = now_epoch_seconds() as i64;

    // Cutoff in the future: the URL subscription (refreshed ~now) is due.
    let due = db
        .list_refreshable_due_subscriptions(now + 100_000)
        .await
        .expect("due query should succeed");
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].source_url, "https://example.com/due");

    // Cutoff far in the past: nothing is due yet.
    let not_due = db
        .list_refreshable_due_subscriptions(now - 100_000)
        .await
        .expect("due query should succeed");
    assert!(not_due.is_empty());

    let _ = std::fs::remove_file(db_path);
}
