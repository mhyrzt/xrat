use std::path::Path;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::app::daemon::server::{
    DaemonRequest, DaemonRequestKind, DaemonResponse, DaemonShutdownPayload, PROTOCOL_VERSION,
    PingPayload, ProxyControlPayload, ProxyStatusPayload, RotationTrigger, RuntimeConnectPayload,
    RuntimeDisconnectPayload, RuntimeReplacePayload, RuntimeStatusPayload,
};

pub async fn ping_daemon(socket_path: &Path) -> crate::app::Result<DaemonResponse<PingPayload>> {
    request_response(socket_path, DaemonRequestKind::DaemonPing).await
}

pub async fn runtime_status_daemon(
    socket_path: &Path,
) -> crate::app::Result<DaemonResponse<RuntimeStatusPayload>> {
    request_response(socket_path, DaemonRequestKind::RuntimeStatus).await
}

pub async fn runtime_connect_daemon(
    socket_path: &Path,
    config_id: i64,
) -> crate::app::Result<DaemonResponse<RuntimeConnectPayload>> {
    request_response(socket_path, DaemonRequestKind::RuntimeConnect { config_id }).await
}

pub async fn runtime_disconnect_daemon(
    socket_path: &Path,
) -> crate::app::Result<DaemonResponse<RuntimeDisconnectPayload>> {
    request_response(socket_path, DaemonRequestKind::RuntimeDisconnect).await
}

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

pub async fn daemon_shutdown_daemon(
    socket_path: &Path,
) -> crate::app::Result<DaemonResponse<DaemonShutdownPayload>> {
    request_response(socket_path, DaemonRequestKind::DaemonShutdown).await
}

pub async fn proxy_start_daemon(
    socket_path: &Path,
) -> crate::app::Result<DaemonResponse<ProxyControlPayload>> {
    request_response(socket_path, DaemonRequestKind::ProxyStart).await
}

pub async fn proxy_status_daemon(
    socket_path: &Path,
) -> crate::app::Result<DaemonResponse<ProxyStatusPayload>> {
    request_response(socket_path, DaemonRequestKind::ProxyStatus).await
}

pub async fn proxy_stop_daemon(
    socket_path: &Path,
) -> crate::app::Result<DaemonResponse<ProxyControlPayload>> {
    request_response(socket_path, DaemonRequestKind::ProxyStop).await
}

async fn request_response<T: serde::de::DeserializeOwned>(
    socket_path: &Path,
    request_kind: DaemonRequestKind,
) -> crate::app::Result<T> {
    let response_bytes = send_request(socket_path, request_kind).await?;
    Ok(serde_json::from_slice::<T>(&response_bytes)?)
}

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
