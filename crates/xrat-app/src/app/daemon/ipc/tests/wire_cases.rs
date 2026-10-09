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

#[tokio::test]
async fn tun_request_roundtrips_mode_and_config_path_and_propagates_failure() {
    let socket_path = test_socket_path("tun-roundtrip");
    let (tx, mut rx) = mpsc::channel(8);
    let supervisor_task = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                crate::app::daemon::supervisor::SupervisorEvent::RuntimeTun {
                    enabled,
                    config_path,
                    respond_to,
                } => {
                    assert_eq!(
                        config_path,
                        std::path::PathBuf::from("/tmp/custom-xrat.toml")
                    );
                    if enabled == Some(true) {
                        let _ = respond_to.send(Ok(crate::app::daemon::ipc::TunStatePayload {
                            enabled: true,
                            active: true,
                            engine: "xray".into(),
                            interface: Some("xrat0".into()),
                            active_config_ref: Some("abc123".into()),
                            session_id: Some(42),
                            split_mode: "all".into(),
                            blacklist_count: 0,
                            whitelist_count: 0,
                        }));
                    } else {
                        let _ = respond_to
                            .send(Err("preflight rejected; current connection kept".into()));
                    }
                }
                crate::app::daemon::supervisor::SupervisorEvent::DaemonShutdown { respond_to } => {
                    let _ =
                        respond_to.send(crate::app::daemon::supervisor::DaemonShutdownResult::Ok(
                            crate::app::daemon::ipc::DaemonShutdownPayload {
                                daemon_ready: false,
                                runtime_disconnected: false,
                            },
                        ));
                    break;
                }
                _ => {}
            }
        }
    });
    let server_socket = socket_path.clone();
    let server_task = tokio::spawn(async move { serve_ping(&server_socket, tx).await });
    for _ in 0..50 {
        if socket_path.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let enabled = crate::app::daemon::ipc::runtime_tun_daemon(
        &socket_path,
        Some(true),
        "/tmp/custom-xrat.toml".into(),
    )
    .await
    .unwrap();
    assert!(enabled.ok);
    let state = enabled.payload.unwrap();
    assert!(state.enabled && state.active);
    assert_eq!(state.active_config_ref.as_deref(), Some("abc123"));
    let failed = crate::app::daemon::ipc::runtime_tun_daemon(
        &socket_path,
        Some(false),
        "/tmp/custom-xrat.toml".into(),
    )
    .await
    .unwrap();
    assert!(!failed.ok);
    assert!(failed.message.contains("current connection kept"));
    assert!(failed.payload.is_none());
    shutdown_test_server(&socket_path).await;
    server_task.await.unwrap().unwrap();
    supervisor_task.await.unwrap();
}

#[test]
fn old_ping_does_not_claim_live_tun_support() {
    let ping: crate::app::daemon::ipc::PingPayload =
        serde_json::from_str(r#"{"daemon_ready":true}"#).unwrap();
    assert!(!ping.live_tun);
}
