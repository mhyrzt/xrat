use std::path::Path;

use crate::app::daemon::ipc::{
    DaemonResponse, DaemonShutdownPayload, PingPayload, ProxyControlPayload, ProxyStatusPayload,
    RotationTrigger, RuntimeConnectPayload, RuntimeDisconnectPayload, RuntimeReplacePayload,
    RuntimeStatusPayload,
};

pub async fn ping_daemon(_socket_path: &Path) -> crate::app::Result<DaemonResponse<PingPayload>> {
    unsupported_client()
}

pub async fn runtime_status_daemon(
    _socket_path: &Path,
) -> crate::app::Result<DaemonResponse<RuntimeStatusPayload>> {
    unsupported_client()
}

pub async fn runtime_connect_daemon(
    _socket_path: &Path,
    _config_id: i64,
) -> crate::app::Result<DaemonResponse<RuntimeConnectPayload>> {
    unsupported_client()
}

pub async fn runtime_disconnect_daemon(
    _socket_path: &Path,
) -> crate::app::Result<DaemonResponse<RuntimeDisconnectPayload>> {
    unsupported_client()
}

pub async fn runtime_replace_daemon(
    _socket_path: &Path,
    _trigger: RotationTrigger,
    _candidate_id: Option<i64>,
) -> crate::app::Result<DaemonResponse<RuntimeReplacePayload>> {
    unsupported_client()
}

pub async fn daemon_shutdown_daemon(
    _socket_path: &Path,
) -> crate::app::Result<DaemonResponse<DaemonShutdownPayload>> {
    unsupported_client()
}

pub async fn proxy_start_daemon(
    _socket_path: &Path,
) -> crate::app::Result<DaemonResponse<ProxyControlPayload>> {
    unsupported_client()
}

pub async fn proxy_status_daemon(
    _socket_path: &Path,
) -> crate::app::Result<DaemonResponse<ProxyStatusPayload>> {
    unsupported_client()
}

pub async fn proxy_stop_daemon(
    _socket_path: &Path,
) -> crate::app::Result<DaemonResponse<ProxyControlPayload>> {
    unsupported_client()
}

fn unsupported_client<T>() -> crate::app::Result<T> {
    Err(crate::app::AppError::InvalidArgument(
        "daemon IPC client is not supported on this platform yet".to_string(),
    ))
}
