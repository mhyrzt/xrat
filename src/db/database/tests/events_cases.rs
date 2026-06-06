use super::{Database, test_database_path};
use crate::db::{EventFilter, NewEvent};

fn new_event(level: &str, source: &str, kind: &str) -> NewEvent {
    NewEvent {
        level: level.to_string(),
        source: source.to_string(),
        kind: kind.to_string(),
        config_id: None,
        session_id: None,
        message: format!("{kind} happened"),
        detail: None,
    }
}

#[tokio::test]
async fn records_and_queries_events_newest_first() {
    let db_path = test_database_path("xrat-events-query");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");

    db.record_event(&new_event("info", "runtime", "connect"))
        .await
        .expect("record should succeed");
    db.record_event(&new_event("warn", "rotation", "rotation_failed"))
        .await
        .expect("record should succeed");
    db.record_event(&new_event("error", "runtime", "connect_failed"))
        .await
        .expect("record should succeed");

    let all = db
        .list_events(&EventFilter {
            limit: 10,
            ..EventFilter::default()
        })
        .await
        .expect("query should succeed");
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].kind, "connect_failed"); // newest first

    let source_filtered = db
        .list_events(&EventFilter {
            source: Some("rotation".to_string()),
            limit: 10,
            ..EventFilter::default()
        })
        .await
        .expect("query should succeed");
    assert_eq!(source_filtered.len(), 1);
    assert_eq!(source_filtered[0].source, "rotation");

    let warn_and_above = db
        .list_events(&EventFilter {
            levels: Some(vec!["warn".to_string(), "error".to_string()]),
            limit: 10,
            ..EventFilter::default()
        })
        .await
        .expect("query should succeed");
    assert_eq!(warn_and_above.len(), 2);
    assert!(warn_and_above.iter().all(|event| event.level != "info"));

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn clear_events_removes_all_persisted_events() {
    let db_path = test_database_path("xrat-events-clear");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");

    db.record_event(&new_event("info", "runtime", "connect"))
        .await
        .expect("record should succeed");
    db.record_event(&new_event("warn", "rotation", "rotation_failed"))
        .await
        .expect("record should succeed");

    let cleared = db.clear_events().await.expect("clear should succeed");
    assert_eq!(cleared, 2);

    let remaining = db
        .list_events(&EventFilter {
            limit: 10,
            ..EventFilter::default()
        })
        .await
        .expect("query should succeed");
    assert!(remaining.is_empty());

    let cleared_again = db.clear_events().await.expect("clear should succeed");
    assert_eq!(cleared_again, 0);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn events_after_returns_only_newer_oldest_first() {
    let db_path = test_database_path("xrat-events-after");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");

    let first = db
        .record_event(&new_event("info", "daemon", "daemon_started"))
        .await
        .expect("record should succeed");
    db.record_event(&new_event("info", "runtime", "connect"))
        .await
        .expect("record should succeed");
    db.record_event(&new_event("info", "runtime", "disconnect"))
        .await
        .expect("record should succeed");

    let after = db
        .events_after(first, &EventFilter::default())
        .await
        .expect("query should succeed");
    assert_eq!(after.len(), 2);
    assert_eq!(after[0].kind, "connect"); // oldest first
    assert_eq!(after[1].kind, "disconnect");
    assert!(after.iter().all(|event| event.id > first));

    let _ = std::fs::remove_file(db_path);
}
