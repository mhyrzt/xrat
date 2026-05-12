use std::path::Path;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(unix)]
use tokio::net::UnixStream;

use crate::app::daemon::server::{
    DaemonRequest, DaemonRequestKind, DaemonResponse, DaemonShutdownPayload, PROTOCOL_VERSION,
    PingPayload, ProxyControlPayload, ProxyStatusPayload, RotationTrigger, RuntimeConnectPayload,
    RuntimeDisconnectPayload, RuntimeReplacePayload, RuntimeStatusPayload,
};

#[cfg(unix)]
pub async fn ping_daemon(socket_path: &Path) -> crate::app::Result<DaemonResponse<PingPayload>> {
    request_response(socket_path, DaemonRequestKind::DaemonPing).await
}

#[cfg(unix)]
pub async fn runtime_status_daemon(
    socket_path: &Path,
) -> crate::app::Result<DaemonResponse<RuntimeStatusPayload>> {
    request_response(socket_path, DaemonRequestKind::RuntimeStatus).await
}

#[cfg(unix)]
pub async fn runtime_connect_daemon(
    socket_path: &Path,
    config_id: i64,
) -> crate::app::Result<DaemonResponse<RuntimeConnectPayload>> {
    request_response(socket_path, DaemonRequestKind::RuntimeConnect { config_id }).await
}

#[cfg(unix)]
pub async fn runtime_disconnect_daemon(
    socket_path: &Path,
) -> crate::app::Result<DaemonResponse<RuntimeDisconnectPayload>> {
    request_response(socket_path, DaemonRequestKind::RuntimeDisconnect).await
}

#[cfg(unix)]
pub async fn runtime_replace_daemon(
    socket_path: &Path,
    trigger: RotationTrigger,
    candidate_id: Option<i64>,
) -> crate::app::Result<DaemonResponse<RuntimeReplacePayload>> {
    request_response(
        socket_path,
        DaemonRequestKind::RuntimeReplace {
            trigger,
            candidate_id,
        },
    )
    .await
}

#[cfg(unix)]
pub async fn daemon_shutdown_daemon(
    socket_path: &Path,
) -> crate::app::Result<DaemonResponse<DaemonShutdownPayload>> {
    request_response(socket_path, DaemonRequestKind::DaemonShutdown).await
}

#[cfg(unix)]
pub async fn proxy_start_daemon(
    socket_path: &Path,
) -> crate::app::Result<DaemonResponse<ProxyControlPayload>> {
    request_response(socket_path, DaemonRequestKind::ProxyStart).await
}

#[cfg(unix)]
pub async fn proxy_status_daemon(
    socket_path: &Path,
) -> crate::app::Result<DaemonResponse<ProxyStatusPayload>> {
    request_response(socket_path, DaemonRequestKind::ProxyStatus).await
}

#[cfg(unix)]
pub async fn proxy_stop_daemon(
    socket_path: &Path,
) -> crate::app::Result<DaemonResponse<ProxyControlPayload>> {
    request_response(socket_path, DaemonRequestKind::ProxyStop).await
}

#[cfg(unix)]
async fn request_response<T: serde::de::DeserializeOwned>(
    socket_path: &Path,
    request_kind: DaemonRequestKind,
) -> crate::app::Result<T> {
    let response_bytes = send_request(socket_path, request_kind).await?;
    Ok(serde_json::from_slice::<T>(&response_bytes)?)
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
pub async fn ping_daemon(_socket_path: &Path) -> crate::app::Result<DaemonResponse<PingPayload>> {
    unsupported_client()
}

#[cfg(not(unix))]
pub async fn runtime_status_daemon(
    _socket_path: &Path,
) -> crate::app::Result<DaemonResponse<RuntimeStatusPayload>> {
    unsupported_client()
}

#[cfg(not(unix))]
pub async fn runtime_connect_daemon(
    _socket_path: &Path,
    _config_id: i64,
) -> crate::app::Result<DaemonResponse<RuntimeConnectPayload>> {
    unsupported_client()
}

#[cfg(not(unix))]
pub async fn runtime_disconnect_daemon(
    _socket_path: &Path,
) -> crate::app::Result<DaemonResponse<RuntimeDisconnectPayload>> {
    unsupported_client()
}

#[cfg(not(unix))]
pub async fn runtime_replace_daemon(
    _socket_path: &Path,
    _trigger: RotationTrigger,
    _candidate_id: Option<i64>,
) -> crate::app::Result<DaemonResponse<RuntimeReplacePayload>> {
    unsupported_client()
}

#[cfg(not(unix))]
pub async fn daemon_shutdown_daemon(
    _socket_path: &Path,
) -> crate::app::Result<DaemonResponse<DaemonShutdownPayload>> {
    unsupported_client()
}

#[cfg(not(unix))]
pub async fn proxy_start_daemon(
    _socket_path: &Path,
) -> crate::app::Result<DaemonResponse<ProxyControlPayload>> {
    unsupported_client()
}

#[cfg(not(unix))]
pub async fn proxy_status_daemon(
    _socket_path: &Path,
) -> crate::app::Result<DaemonResponse<ProxyStatusPayload>> {
    unsupported_client()
}

#[cfg(not(unix))]
pub async fn proxy_stop_daemon(
    _socket_path: &Path,
) -> crate::app::Result<DaemonResponse<ProxyControlPayload>> {
    unsupported_client()
}

#[cfg(not(unix))]
fn unsupported_client<T>() -> crate::app::Result<T> {
    Err(crate::app::AppError::InvalidArgument(
        "daemon IPC client is not supported on this platform yet".to_string(),
    ))
}
