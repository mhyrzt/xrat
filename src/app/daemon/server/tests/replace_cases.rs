use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::timeout;

use super::{
    shutdown_test_server, spawn_test_supervisor, spawn_test_supervisor_replace_error,
    test_socket_path, wait_until_reachable,
};
use crate::app::daemon::server::{
    DaemonResponseCode, RotationTrigger, runtime_replace_daemon, serve_ping,
};

#[tokio::test]
async fn runtime_replace_request_returns_payload() {
    let socket_path = test_socket_path("runtime-replace-ok");
    let _ = std::fs::remove_file(&socket_path);
    let (tx, rx) = mpsc::channel(8);
    let supervisor_task = spawn_test_supervisor(rx);
    let server_socket = socket_path.clone();
    let server_task = tokio::spawn(async move { serve_ping(&server_socket, tx).await });
    wait_until_reachable(&socket_path).await;

    let response = runtime_replace_daemon(&socket_path, RotationTrigger::Manual, Some(99))
        .await
        .expect("replace request should succeed");
    assert!(response.ok);
    assert!(matches!(response.code, DaemonResponseCode::Ok));
    assert_eq!(response.message, "runtime replaced");
    let payload = response.payload.expect("replace payload should exist");
    assert!(matches!(payload.trigger, RotationTrigger::Manual));
    assert!(payload.replaced);
    assert_eq!(payload.old_session_id, 10);
    assert_eq!(payload.new_config_id, 99);
    assert_eq!(payload.new_session_id, 30);
    assert_eq!(payload.new_pid, 40);

    let _ = shutdown_test_server(&socket_path).await;
    let _ = timeout(Duration::from_secs(1), server_task).await;
    let _ = timeout(Duration::from_secs(1), supervisor_task).await;
}

#[tokio::test]
async fn runtime_replace_request_maps_supervisor_error() {
    let socket_path = test_socket_path("runtime-replace-error");
    let _ = std::fs::remove_file(&socket_path);
    let (tx, rx) = mpsc::channel(8);
    let supervisor_task = spawn_test_supervisor_replace_error(rx);
    let server_socket = socket_path.clone();
    let server_task = tokio::spawn(async move { serve_ping(&server_socket, tx).await });
    wait_until_reachable(&socket_path).await;

    let response = runtime_replace_daemon(&socket_path, RotationTrigger::Manual, None)
        .await
        .expect("replace request should return response envelope");
    assert!(!response.ok);
    assert!(matches!(response.code, DaemonResponseCode::InvalidState));
    assert_eq!(response.message, "replace validation failed");
    assert!(response.payload.is_none());

    let _ = shutdown_test_server(&socket_path).await;
    let _ = timeout(Duration::from_secs(1), server_task).await;
    let _ = timeout(Duration::from_secs(1), supervisor_task).await;
}
