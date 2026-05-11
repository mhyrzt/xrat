use super::support::{import_single_config, write_fake_runtime_script};
use super::*;

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
