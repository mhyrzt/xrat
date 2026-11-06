use super::support::{import_hy2_config, import_single_config, write_fake_runtime_script};
use super::*;

#[tokio::test]
async fn status_prefers_running_session_over_future_dated_stopped_session() {
    let context = test_context().await;
    let config = import_single_config(&context).await;
    let session = |status, process_id, config_id| RuntimeSessionInsert {
        config_id,
        status,
        socks_host: None,
        socks_port: None,
        http_host: None,
        http_port: None,
        shadowsocks_host: None,
        shadowsocks_port: None,
        process_id,
        failure_reason: None,
        started_at: None,
        stopped_at: None,
    };
    let stopped_id = context
        .db
        .insert_runtime_session(&session(RuntimeSessionStatus::Stopped, None, None))
        .await
        .unwrap();
    let running_id = context
        .db
        .insert_runtime_session(&session(
            RuntimeSessionStatus::Running,
            Some(i64::from(std::process::id())),
            Some(config.id),
        ))
        .await
        .unwrap();
    let pool = sqlx::SqlitePool::connect(&format!(
        "sqlite:{}",
        context.runtime_paths.database_path.display()
    ))
    .await
    .unwrap();
    sqlx::query("UPDATE runtime_sessions SET created_at = '9999-01-01 00:00:00' WHERE id = ?")
        .bind(stopped_id)
        .execute(&pool)
        .await
        .unwrap();

    let snapshot = RuntimeService::new(&context).status().await.unwrap();
    assert_eq!(snapshot.session.unwrap().id, running_id);
    assert_eq!(
        snapshot.status,
        RuntimeSessionDisplay::Persisted(RuntimeSessionStatus::Running)
    );
    assert!(snapshot.pid_running);
    assert_eq!(snapshot.session_config.unwrap().id, config.id);
}

#[tokio::test]
async fn connect_and_disconnect_persist_direct_transition_metadata() {
    let mut context = test_context().await;
    let config = import_single_config(&context).await;

    write_fake_runtime_script(&context);
    context.runtime_paths.xray_path = context.runtime_paths.root_dir.join("fake-xray.py");

    let connected = RuntimeService::new(&context)
        .connect(ConnectRequest {
            config_id: config.id,
        })
        .await
        .expect("connect should succeed");
    assert!(connected.pid > 0);

    let connected_session = context
        .db
        .get_running_runtime_session()
        .await
        .expect("running session should load")
        .expect("running session should exist");
    assert_eq!(
        connected_session.last_transition_reason_code.as_deref(),
        Some("manual_connect")
    );
    assert_eq!(
        connected_session.last_transition_reason_detail.as_deref(),
        Some("runtime connect request succeeded")
    );
    assert_eq!(
        connected_session.last_transition_origin.as_deref(),
        Some("cli")
    );
    assert_eq!(connected_session.owner_kind.as_deref(), Some("cli"));

    let disconnected = RuntimeService::new(&context)
        .disconnect()
        .await
        .expect("disconnect should succeed");
    assert!(disconnected.stopped_session);

    let latest = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("latest session should load")
        .expect("latest session should exist");
    assert_eq!(latest.id, connected_session.id);
    assert_eq!(latest.status, RuntimeSessionStatus::Stopped);
    assert_eq!(
        latest.last_transition_reason_code.as_deref(),
        Some("manual_disconnect")
    );
    assert_eq!(
        latest.last_transition_reason_detail.as_deref(),
        Some("runtime disconnect request succeeded")
    );
    assert_eq!(latest.last_transition_origin.as_deref(), Some("cli"));
    assert_eq!(latest.owner_kind.as_deref(), Some("cli"));

    let _ = xray_runtime::terminate_process_gracefully(connected.pid as i64, SHUTDOWN_TIMEOUT);
}

#[tokio::test]
async fn connect_hy2_uses_managed_singbox_runtime() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "sing-box".to_string();
    let config = import_hy2_config(&context).await;

    write_fake_runtime_script(&context);
    context.runtime_paths.sing_box_path = context.runtime_paths.root_dir.join("fake-xray.py");

    let connected = RuntimeService::new(&context)
        .connect(ConnectRequest {
            config_id: config.id,
        })
        .await
        .expect("hy2 connect should launch sing-box runtime");
    assert!(connected.pid > 0);
    assert!(
        connected
            .runtime_config_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".singbox.json"))
    );

    let running = context
        .db
        .get_running_runtime_session()
        .await
        .expect("running session should load")
        .expect("running session should exist");
    assert_eq!(running.config_id, Some(config.id));
    assert_eq!(running.status, RuntimeSessionStatus::Running);
    assert_eq!(
        running.socks_port,
        Some(i64::from(context.app_config.runtime.socks.port))
    );

    RuntimeService::new(&context)
        .disconnect()
        .await
        .expect("disconnect should stop sing-box runtime");

    let _ = xray_runtime::terminate_process_gracefully(connected.pid as i64, SHUTDOWN_TIMEOUT);
}
