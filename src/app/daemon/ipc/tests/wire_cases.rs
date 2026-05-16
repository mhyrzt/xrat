use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tokio::time::timeout;

use super::{shutdown_test_server, spawn_test_supervisor, test_socket_path, wait_until_reachable};
use crate::app::daemon::ipc::{
    DaemonRequest, DaemonRequestKind, DaemonResponse, DaemonResponseCode, PROTOCOL_VERSION,
    serve_ping,
};

#[tokio::test]
async fn rejects_incompatible_protocol_version() {
    let socket_path = test_socket_path("protocol-mismatch");
    let _ = std::fs::remove_file(&socket_path);
    let (tx, rx) = mpsc::channel(8);
    let supervisor_task = spawn_test_supervisor(rx);
    let server_socket = socket_path.clone();
    let server_task = tokio::spawn(async move { serve_ping(&server_socket, tx).await });
    wait_until_reachable(&socket_path).await;

    let mut stream = UnixStream::connect(&socket_path)
        .await
        .expect("connect should succeed");
    let request = DaemonRequest {
        protocol_version: PROTOCOL_VERSION + 1,
        request: DaemonRequestKind::DaemonPing,
    };
    let encoded = serde_json::to_vec(&request).expect("request serialization should succeed");
    stream
        .write_all(&encoded)
        .await
        .expect("write should succeed");
    stream.shutdown().await.expect("shutdown should succeed");

    let mut response_bytes = Vec::new();
    stream
        .read_to_end(&mut response_bytes)
        .await
        .expect("read should succeed");
    let response = serde_json::from_slice::<DaemonResponse<serde_json::Value>>(&response_bytes)
        .expect("response parse should succeed");
    assert!(!response.ok);
    assert!(matches!(response.code, DaemonResponseCode::InvalidState));
    assert!(response.message.contains("unsupported protocol version"));

    let _ = shutdown_test_server(&socket_path).await;
    let _ = timeout(Duration::from_secs(1), server_task).await;
    let _ = timeout(Duration::from_secs(1), supervisor_task).await;
}
