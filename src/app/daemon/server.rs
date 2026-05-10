use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, oneshot};

use crate::app::daemon::supervisor::{
    DaemonShutdownResult, RuntimeConnectResult, RuntimeStatusResult, SupervisorEvent,
};

pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonRequest {
    pub protocol_version: u16,
    pub request: DaemonRequestKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonRequestKind {
    DaemonPing,
    DaemonShutdown,
    RuntimeStatus,
    RuntimeConnect { config_id: i64 },
    RuntimeDisconnect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonResponse<T> {
    pub protocol_version: u16,
    pub ok: bool,
    pub code: DaemonResponseCode,
    pub message: String,
    pub payload: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DaemonResponseCode {
    Ok,
    Busy,
    NotFound,
    InvalidState,
    InternalError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingPayload {
    pub daemon_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatusPayload {
    pub daemon_ready: bool,
    pub runtime_owned: bool,
    pub runtime_status: String,
    pub session_id: Option<i64>,
    pub active_config_id: Option<i64>,
    pub pid_running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConnectPayload {
    pub config_id: i64,
    pub session_id: i64,
    pub pid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDisconnectPayload {
    pub stopped_session: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonShutdownPayload {
    pub daemon_ready: bool,
    pub runtime_disconnected: bool,
}

pub fn default_socket_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("daemon.sock")
}

pub fn ping_response() -> DaemonResponse<PingPayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: true,
        code: DaemonResponseCode::Ok,
        message: "daemon reachable".to_string(),
        payload: Some(PingPayload { daemon_ready: true }),
    }
}

pub fn runtime_status_response(
    payload: RuntimeStatusPayload,
) -> DaemonResponse<RuntimeStatusPayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: true,
        code: DaemonResponseCode::Ok,
        message: "runtime status available".to_string(),
        payload: Some(payload),
    }
}

pub fn runtime_status_error_response(message: String) -> DaemonResponse<RuntimeStatusPayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: false,
        code: DaemonResponseCode::InternalError,
        message,
        payload: None,
    }
}

pub fn runtime_connect_response(
    payload: RuntimeConnectPayload,
) -> DaemonResponse<RuntimeConnectPayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: true,
        code: DaemonResponseCode::Ok,
        message: "runtime connected".to_string(),
        payload: Some(payload),
    }
}

pub fn runtime_connect_error_response(message: String) -> DaemonResponse<RuntimeConnectPayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: false,
        code: DaemonResponseCode::InvalidState,
        message,
        payload: None,
    }
}

pub fn runtime_disconnect_response(
    payload: RuntimeDisconnectPayload,
) -> DaemonResponse<RuntimeDisconnectPayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: true,
        code: DaemonResponseCode::Ok,
        message: "runtime disconnected".to_string(),
        payload: Some(payload),
    }
}

pub fn runtime_disconnect_error_response(
    message: String,
) -> DaemonResponse<RuntimeDisconnectPayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: false,
        code: DaemonResponseCode::InvalidState,
        message,
        payload: None,
    }
}

pub fn daemon_shutdown_response(
    payload: DaemonShutdownPayload,
) -> DaemonResponse<DaemonShutdownPayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: true,
        code: DaemonResponseCode::Ok,
        message: "daemon shutdown requested".to_string(),
        payload: Some(payload),
    }
}

#[cfg(unix)]
pub async fn serve_ping(
    socket_path: &Path,
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<()> {
    if let Some(parent) = socket_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    if socket_path.exists() {
        match ping_daemon(socket_path).await {
            Ok(_) => {
                return Err(crate::app::AppError::InvalidArgument(format!(
                    "daemon is already running at {}; stop it first",
                    socket_path.display()
                )));
            }
            Err(err) if daemon_unreachable(&err) => {
                let _ = std::fs::remove_file(socket_path);
            }
            Err(err) => return Err(err),
        }
    }

    let listener = UnixListener::bind(socket_path)?;
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    loop {
        tokio::select! {
            biased;
            _ = shutdown_rx.recv() => {
                break;
            }
            accept_result = listener.accept() => {
                let (mut stream, _) = accept_result?;
                let supervisor_tx = supervisor_tx.clone();
                let shutdown_tx = shutdown_tx.clone();
                tokio::spawn(async move {
                    let _ = handle_connection(&mut stream, supervisor_tx, shutdown_tx).await;
                });
            }
        }
    }
    let _ = std::fs::remove_file(socket_path);
    Ok(())
}

#[cfg(unix)]
pub async fn ping_daemon(socket_path: &Path) -> crate::app::Result<DaemonResponse<PingPayload>> {
    let response_bytes = send_request(socket_path, DaemonRequestKind::DaemonPing).await?;
    let response = serde_json::from_slice::<DaemonResponse<PingPayload>>(&response_bytes)?;
    Ok(response)
}

#[cfg(unix)]
pub async fn runtime_status_daemon(
    socket_path: &Path,
) -> crate::app::Result<DaemonResponse<RuntimeStatusPayload>> {
    let response_bytes = send_request(socket_path, DaemonRequestKind::RuntimeStatus).await?;
    let response = serde_json::from_slice::<DaemonResponse<RuntimeStatusPayload>>(&response_bytes)?;
    Ok(response)
}

#[cfg(unix)]
pub async fn runtime_connect_daemon(
    socket_path: &Path,
    config_id: i64,
) -> crate::app::Result<DaemonResponse<RuntimeConnectPayload>> {
    let response_bytes =
        send_request(socket_path, DaemonRequestKind::RuntimeConnect { config_id }).await?;
    let response =
        serde_json::from_slice::<DaemonResponse<RuntimeConnectPayload>>(&response_bytes)?;
    Ok(response)
}

#[cfg(unix)]
pub async fn runtime_disconnect_daemon(
    socket_path: &Path,
) -> crate::app::Result<DaemonResponse<RuntimeDisconnectPayload>> {
    let response_bytes = send_request(socket_path, DaemonRequestKind::RuntimeDisconnect).await?;
    let response =
        serde_json::from_slice::<DaemonResponse<RuntimeDisconnectPayload>>(&response_bytes)?;
    Ok(response)
}

#[cfg(unix)]
pub async fn daemon_shutdown_daemon(
    socket_path: &Path,
) -> crate::app::Result<DaemonResponse<DaemonShutdownPayload>> {
    let response_bytes = send_request(socket_path, DaemonRequestKind::DaemonShutdown).await?;
    let response =
        serde_json::from_slice::<DaemonResponse<DaemonShutdownPayload>>(&response_bytes)?;
    Ok(response)
}

#[cfg(unix)]
async fn send_request(
    socket_path: &Path,
    request_kind: DaemonRequestKind,
) -> crate::app::Result<Vec<u8>> {
    let mut stream = UnixStream::connect(socket_path).await?;
    let request = DaemonRequest {
        protocol_version: PROTOCOL_VERSION,
        request: request_kind,
    };
    let mut encoded = serde_json::to_vec(&request)?;
    encoded.push(b'\n');
    stream.write_all(&encoded).await?;
    stream.shutdown().await?;

    let mut response_bytes = Vec::new();
    stream.read_to_end(&mut response_bytes).await?;
    Ok(response_bytes)
}

#[cfg(not(unix))]
pub async fn serve_ping(
    _socket_path: &Path,
    _supervisor_tx: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<()> {
    Err(crate::app::AppError::InvalidArgument(
        "daemon IPC server is not supported on this platform yet".to_string(),
    ))
}

#[cfg(not(unix))]
pub async fn ping_daemon(_socket_path: &Path) -> crate::app::Result<DaemonResponse<PingPayload>> {
    Err(crate::app::AppError::InvalidArgument(
        "daemon IPC client is not supported on this platform yet".to_string(),
    ))
}

#[cfg(not(unix))]
pub async fn runtime_status_daemon(
    _socket_path: &Path,
) -> crate::app::Result<DaemonResponse<RuntimeStatusPayload>> {
    Err(crate::app::AppError::InvalidArgument(
        "daemon IPC client is not supported on this platform yet".to_string(),
    ))
}

#[cfg(not(unix))]
pub async fn runtime_connect_daemon(
    _socket_path: &Path,
    _config_id: i64,
) -> crate::app::Result<DaemonResponse<RuntimeConnectPayload>> {
    Err(crate::app::AppError::InvalidArgument(
        "daemon IPC client is not supported on this platform yet".to_string(),
    ))
}

#[cfg(not(unix))]
pub async fn runtime_disconnect_daemon(
    _socket_path: &Path,
) -> crate::app::Result<DaemonResponse<RuntimeDisconnectPayload>> {
    Err(crate::app::AppError::InvalidArgument(
        "daemon IPC client is not supported on this platform yet".to_string(),
    ))
}

#[cfg(not(unix))]
pub async fn daemon_shutdown_daemon(
    _socket_path: &Path,
) -> crate::app::Result<DaemonResponse<DaemonShutdownPayload>> {
    Err(crate::app::AppError::InvalidArgument(
        "daemon IPC client is not supported on this platform yet".to_string(),
    ))
}

#[cfg(unix)]
async fn handle_connection(
    stream: &mut UnixStream,
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
    shutdown_tx: mpsc::Sender<()>,
) -> crate::app::Result<()> {
    let mut request_bytes = Vec::new();
    stream.read_to_end(&mut request_bytes).await?;
    let request = serde_json::from_slice::<DaemonRequest>(&request_bytes)?;
    if request.protocol_version != PROTOCOL_VERSION {
        let response = DaemonResponse::<serde_json::Value> {
            protocol_version: PROTOCOL_VERSION,
            ok: false,
            code: DaemonResponseCode::InvalidState,
            message: format!(
                "unsupported protocol version {} (expected {})",
                request.protocol_version, PROTOCOL_VERSION
            ),
            payload: None,
        };
        stream.write_all(&serde_json::to_vec(&response)?).await?;
        return Ok(());
    }

    let (encoded, should_shutdown) = match request.request {
        DaemonRequestKind::DaemonPing => (
            serde_json::to_vec(&ping_response_via_supervisor(supervisor_tx).await?)?,
            false,
        ),
        DaemonRequestKind::RuntimeStatus => (
            serde_json::to_vec(&runtime_status_response_via_supervisor(supervisor_tx).await?)?,
            false,
        ),
        DaemonRequestKind::RuntimeConnect { config_id } => (
            serde_json::to_vec(
                &runtime_connect_response_via_supervisor(supervisor_tx, config_id).await?,
            )?,
            false,
        ),
        DaemonRequestKind::RuntimeDisconnect => (
            serde_json::to_vec(&runtime_disconnect_response_via_supervisor(supervisor_tx).await?)?,
            false,
        ),
        DaemonRequestKind::DaemonShutdown => (
            serde_json::to_vec(&daemon_shutdown_response_via_supervisor(supervisor_tx).await?)?,
            true,
        ),
    };
    stream.write_all(&encoded).await?;
    if should_shutdown {
        let _ = shutdown_tx.send(()).await;
    }
    Ok(())
}

#[cfg(unix)]
async fn runtime_connect_response_via_supervisor(
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
    config_id: i64,
) -> crate::app::Result<DaemonResponse<RuntimeConnectPayload>> {
    let (tx, rx) = oneshot::channel();
    supervisor_tx
        .send(SupervisorEvent::RuntimeConnect {
            config_id,
            respond_to: tx,
        })
        .await
        .map_err(|_| {
            crate::app::AppError::InvalidArgument("supervisor is not running".to_string())
        })?;
    let payload = rx.await.map_err(|_| {
        crate::app::AppError::InvalidArgument("supervisor response channel closed".to_string())
    })?;
    let response = match payload {
        RuntimeConnectResult::Ok(payload) => runtime_connect_response(payload),
        RuntimeConnectResult::Err { message } => runtime_connect_error_response(message),
    };
    Ok(response)
}

#[cfg(unix)]
async fn runtime_disconnect_response_via_supervisor(
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<DaemonResponse<RuntimeDisconnectPayload>> {
    let (tx, rx) = oneshot::channel();
    supervisor_tx
        .send(SupervisorEvent::RuntimeDisconnect { respond_to: tx })
        .await
        .map_err(|_| {
            crate::app::AppError::InvalidArgument("supervisor is not running".to_string())
        })?;
    let payload = rx.await.map_err(|_| {
        crate::app::AppError::InvalidArgument("supervisor response channel closed".to_string())
    })?;
    let response = match payload {
        crate::app::daemon::supervisor::RuntimeDisconnectResult::Ok(payload) => {
            runtime_disconnect_response(payload)
        }
        crate::app::daemon::supervisor::RuntimeDisconnectResult::Err { message } => {
            runtime_disconnect_error_response(message)
        }
    };
    Ok(response)
}

#[cfg(unix)]
async fn runtime_status_response_via_supervisor(
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<DaemonResponse<RuntimeStatusPayload>> {
    let (tx, rx) = oneshot::channel();
    supervisor_tx
        .send(SupervisorEvent::RuntimeStatus { respond_to: tx })
        .await
        .map_err(|_| {
            crate::app::AppError::InvalidArgument("supervisor is not running".to_string())
        })?;
    let payload = rx.await.map_err(|_| {
        crate::app::AppError::InvalidArgument("supervisor response channel closed".to_string())
    })?;
    let response = match payload {
        RuntimeStatusResult::Ok(payload) => runtime_status_response(payload),
        RuntimeStatusResult::Err { message } => runtime_status_error_response(message),
    };
    Ok(response)
}

#[cfg(unix)]
async fn ping_response_via_supervisor(
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<DaemonResponse<PingPayload>> {
    let (tx, rx) = oneshot::channel();
    supervisor_tx
        .send(SupervisorEvent::DaemonPing { respond_to: tx })
        .await
        .map_err(|_| {
            crate::app::AppError::InvalidArgument("supervisor is not running".to_string())
        })?;
    let payload = rx.await.map_err(|_| {
        crate::app::AppError::InvalidArgument("supervisor response channel closed".to_string())
    })?;

    Ok(DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: true,
        code: DaemonResponseCode::Ok,
        message: "daemon reachable".to_string(),
        payload: Some(payload),
    })
}

#[cfg(unix)]
async fn daemon_shutdown_response_via_supervisor(
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<DaemonResponse<DaemonShutdownPayload>> {
    let (tx, rx) = oneshot::channel();
    supervisor_tx
        .send(SupervisorEvent::DaemonShutdown { respond_to: tx })
        .await
        .map_err(|_| {
            crate::app::AppError::InvalidArgument("supervisor is not running".to_string())
        })?;
    let payload = rx.await.map_err(|_| {
        crate::app::AppError::InvalidArgument("supervisor response channel closed".to_string())
    })?;

    let response = match payload {
        DaemonShutdownResult::Ok(payload) => daemon_shutdown_response(payload),
        DaemonShutdownResult::Err { message } => DaemonResponse {
            protocol_version: PROTOCOL_VERSION,
            ok: false,
            code: DaemonResponseCode::InvalidState,
            message,
            payload: None,
        },
    };
    Ok(response)
}

pub fn daemon_unreachable(err: &crate::app::AppError) -> bool {
    match err {
        crate::app::AppError::Io(io_err) => matches!(
            io_err.kind(),
            ErrorKind::NotFound | ErrorKind::ConnectionRefused | ErrorKind::ConnectionReset
        ),
        _ => false,
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::app::daemon::supervisor::{DaemonShutdownResult, SupervisorEvent};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixStream;
    use tokio::time::timeout;

    fn test_socket_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "xrat-daemon-test-{name}-{}-{stamp}.sock",
            std::process::id()
        ))
    }

    async fn wait_until_reachable(socket_path: &Path) {
        for _ in 0..50 {
            if ping_daemon(socket_path).await.is_ok() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        panic!("daemon socket never became reachable");
    }

    async fn shutdown_test_server(socket_path: &Path) -> DaemonResponse<DaemonShutdownPayload> {
        daemon_shutdown_daemon(socket_path)
            .await
            .expect("shutdown request should succeed")
    }

    fn spawn_test_supervisor(
        mut rx: mpsc::Receiver<SupervisorEvent>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    SupervisorEvent::DaemonPing { respond_to } => {
                        let _ = respond_to.send(PingPayload { daemon_ready: true });
                    }
                    SupervisorEvent::DaemonShutdown { respond_to } => {
                        let _ = respond_to.send(DaemonShutdownResult::Ok(DaemonShutdownPayload {
                            daemon_ready: false,
                            runtime_disconnected: false,
                        }));
                        break;
                    }
                    _ => {}
                }
            }
        })
    }

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
}
