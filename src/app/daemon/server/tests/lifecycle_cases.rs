use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::timeout;

use super::{shutdown_test_server, spawn_test_supervisor, test_socket_path, wait_until_reachable};
use crate::app::daemon::server::{DaemonResponseCode, serve_ping};

#[tokio::test]
async fn shutdown_request_returns_payload_and_stops_server() {
    let socket_path = test_socket_path("shutdown");
    let _ = std::fs::remove_file(&socket_path);
    let (tx, rx) = mpsc::channel(8);
    let supervisor_task = spawn_test_supervisor(rx);
    let server_socket = socket_path.clone();
    let server_task = tokio::spawn(async move { serve_ping(&server_socket, tx).await });

    wait_until_reachable(&socket_path).await;

    let response = shutdown_test_server(&socket_path).await;
    assert!(response.ok);
    assert!(matches!(response.code, DaemonResponseCode::Ok));
    assert_eq!(response.message, "daemon shutdown requested");
    let payload = response.payload.expect("shutdown payload must exist");
    assert!(!payload.daemon_ready);
    assert!(!payload.runtime_disconnected);

    let server_result = timeout(Duration::from_secs(1), server_task)
        .await
        .expect("server should stop after shutdown request");
    server_result
        .expect("server task should join cleanly")
        .expect("server should return Ok");

    let _ = timeout(Duration::from_secs(1), supervisor_task).await;
    assert!(
        !socket_path.exists(),
        "socket should be cleaned up on shutdown"
    );
}

#[tokio::test]
async fn startup_fails_when_existing_socket_is_reachable() {
    let socket_path = test_socket_path("already-running");
    let _ = std::fs::remove_file(&socket_path);

    let (tx1, rx1) = mpsc::channel(8);
    let supervisor_task = spawn_test_supervisor(rx1);
    let server_socket = socket_path.clone();
    let server_task = tokio::spawn(async move { serve_ping(&server_socket, tx1).await });
    wait_until_reachable(&socket_path).await;

    let (tx2, _rx2) = mpsc::channel(8);
    let second_start = serve_ping(&socket_path, tx2).await;
    match second_start {
        Err(crate::app::AppError::InvalidArgument(message)) => {
            assert!(message.contains("already running"));
        }
        other => panic!("expected already-running error, got {other:?}"),
    }

    let _ = shutdown_test_server(&socket_path).await;
    let _ = timeout(Duration::from_secs(1), server_task).await;
    let _ = timeout(Duration::from_secs(1), supervisor_task).await;
}
