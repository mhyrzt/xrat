use std::path::Path;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;

use crate::app::daemon::server::bridge::{
    daemon_shutdown_response_via_supervisor, ping_response_via_supervisor,
    proxy_start_response_via_supervisor, proxy_status_response_via_supervisor,
    proxy_stop_response_via_supervisor, runtime_connect_response_via_supervisor,
    runtime_disconnect_response_via_supervisor, runtime_replace_response_via_supervisor,
    runtime_status_response_via_supervisor,
};
use crate::app::daemon::server::{
    DaemonRequest, DaemonRequestKind, DaemonResponse, DaemonResponseCode, PROTOCOL_VERSION,
    daemon_unreachable,
};
use crate::app::daemon::supervisor::SupervisorEvent;

#[cfg(unix)]
pub async fn serve_ping(
    socket_path: &Path,
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<()> {
    if let Some(parent) = socket_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    if socket_path.exists() {
        match crate::app::daemon::server::ping_daemon(socket_path).await {
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
            _ = shutdown_rx.recv() => break,
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

    let (encoded, should_shutdown) = dispatch_request(request.request, supervisor_tx).await?;
    stream.write_all(&encoded).await?;
    if should_shutdown {
        let _ = shutdown_tx.send(()).await;
    }
    Ok(())
}

#[cfg(unix)]
async fn dispatch_request(
    request: DaemonRequestKind,
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<(Vec<u8>, bool)> {
    let response = match request {
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
        DaemonRequestKind::RuntimeReplace {
            trigger,
            candidate_id,
        } => (
            serde_json::to_vec(
                &runtime_replace_response_via_supervisor(supervisor_tx, trigger, candidate_id)
                    .await?,
            )?,
            false,
        ),
        DaemonRequestKind::DaemonShutdown => (
            serde_json::to_vec(&daemon_shutdown_response_via_supervisor(supervisor_tx).await?)?,
            true,
        ),
        DaemonRequestKind::ProxyStart => (
            serde_json::to_vec(&proxy_start_response_via_supervisor(supervisor_tx).await?)?,
            false,
        ),
        DaemonRequestKind::ProxyStatus => (
            serde_json::to_vec(&proxy_status_response_via_supervisor(supervisor_tx).await?)?,
            false,
        ),
        DaemonRequestKind::ProxyStop => (
            serde_json::to_vec(&proxy_stop_response_via_supervisor(supervisor_tx).await?)?,
            false,
        ),
    };
    Ok(response)
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
