use std::path::Path;

#[cfg(unix)]
use tokio::net::UnixListener;
use tokio::sync::mpsc;

use crate::app::daemon::ipc::daemon_unreachable;
use crate::app::daemon::supervisor::SupervisorEvent;

#[cfg(unix)]
mod dispatch;
#[cfg(unix)]
mod io;

#[cfg(unix)]
pub async fn serve_ping(
    socket_path: &Path,
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<()> {
    if let Some(parent) = socket_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    if socket_path.exists() {
        match crate::app::daemon::ipc::ping_daemon(socket_path).await {
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
                    let _ = io::handle_connection(&mut stream, supervisor_tx, shutdown_tx).await;
                });
            }
        }
    }
    let _ = std::fs::remove_file(socket_path);
    Ok(())
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
