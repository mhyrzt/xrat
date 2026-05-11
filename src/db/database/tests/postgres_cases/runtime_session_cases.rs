use super::*;

pub(super) async fn verify_runtime_session_state(db: &Database, config_id: i64) {
    let session_id = db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config_id),
            status: RuntimeSessionStatus::Starting,
            socks_host: Some("127.0.0.1".to_string()),
            socks_port: Some(10808),
            http_host: Some("127.0.0.1".to_string()),
            http_port: Some(18080),
            shadowsocks_host: Some("127.0.0.1".to_string()),
            shadowsocks_port: Some(1081),
            process_id: None,
            failure_reason: None,
            started_at: Some("2025-01-01T10:00:00Z".to_string()),
            stopped_at: None,
        })
        .await
        .expect("runtime session insert should succeed");
    db.update_runtime_session_state(
        session_id,
        RuntimeSessionStatus::Running,
        Some(4242),
        None,
        None,
        None,
    )
    .await
    .expect("runtime session update should succeed");
    db.mark_runtime_session_stopped(session_id, Some("2025-01-01T10:05:00Z"))
        .await
        .expect("runtime session stop should succeed");

    let latest_session = db
        .get_latest_runtime_session()
        .await
        .expect("latest session should load")
        .expect("latest session should exist");
    assert_eq!(db.get_runtime_session_count().await.expect("count"), 1);
    assert_eq!(latest_session.status, RuntimeSessionStatus::Stopped);
    assert_eq!(latest_session.process_id, Some(4242));
    assert_eq!(latest_session.socks_port, Some(10808));
    assert_eq!(latest_session.http_port, Some(18080));
    assert_eq!(latest_session.shadowsocks_port, Some(1081));
    assert_eq!(
        latest_session.stopped_at.as_deref(),
        Some("2025-01-01T10:05:00Z")
    );
}
