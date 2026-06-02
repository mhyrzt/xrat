use crate::app::commands::daemon_install;
use crate::app::context::AppContext;
use crate::app::daemon::{ipc, supervisor};
use crate::cli::{DaemonAction, DaemonArgs};
use std::process::Stdio;
use tokio::time::{Duration, sleep};

pub async fn run(context: &AppContext, args: &DaemonArgs) -> crate::app::Result<()> {
    let socket_path = ipc::default_socket_path(&context.runtime_paths.runtime_dir);

    match &args.action {
        DaemonAction::Start(_) => {
            if ipc::ping_daemon(&socket_path).await.is_ok() {
                println!("Daemon already running. Socket: {}", socket_path.display());
                return Ok(());
            }
            spawn_detached_daemon(context)?;
            wait_until_daemon_ready(&socket_path).await?;
            println!("Daemon started. Socket: {}", socket_path.display());
        }
        DaemonAction::RunServer(_) => {
            let (tx, rx) = supervisor::channel(32);
            let supervisor_context = context.clone();
            tokio::spawn(supervisor::run(rx, supervisor_context));

            let http_handle = if context.app_config.server.enabled {
                let server_settings = context.app_config.server.clone();
                let db = context.db.clone();
                let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
                let handle = tokio::spawn(async move {
                    if let Err(err) =
                        crate::server::serve_with_shutdown(db, &server_settings, shutdown_rx).await
                    {
                        tracing::error!(error = %err, "HTTP API server failed");
                    }
                });
                Some((handle, shutdown_tx))
            } else {
                None
            };

            ipc::serve_ping(&socket_path, tx).await?;

            if let Some((handle, shutdown_tx)) = http_handle {
                let _ = shutdown_tx.send(());
                let _ = handle.await;
            }
        }
        DaemonAction::Status(_) => match ipc::runtime_status_daemon(&socket_path).await {
            Ok(response) => {
                if !response.ok {
                    return Err(crate::app::AppError::InvalidArgument(response.message));
                }
                let payload = response.payload.unwrap_or(ipc::RuntimeStatusPayload {
                    daemon_ready: false,
                    runtime_owned: false,
                    runtime_status: "unknown".to_string(),
                    session_id: None,
                    active_config_id: None,
                    pid_running: false,
                    http_api_enabled: false,
                    http_api_addr: None,
                });
                println!(
                    "Daemon status: {} (protocol v{}, ready={}, runtime_owned={}, runtime={}, session={:?}, active_config={:?}, pid_running={}, http_api={}, http_addr={})",
                    response.message,
                    response.protocol_version,
                    payload.daemon_ready,
                    payload.runtime_owned,
                    payload.runtime_status,
                    payload.session_id,
                    payload.active_config_id,
                    payload.pid_running,
                    payload.http_api_enabled,
                    payload.http_api_addr.as_deref().unwrap_or("disabled"),
                );
            }
            Err(err) if ipc::daemon_unreachable(&err) => {
                println!(
                    "Daemon status: not running (start with `xrat daemon start`). Socket: {}",
                    socket_path.display()
                );
            }
            Err(err) => return Err(err),
        },
        DaemonAction::Stop(_) => match ipc::daemon_shutdown_daemon(&socket_path).await {
            Ok(response) => {
                if !response.ok {
                    return Err(crate::app::AppError::InvalidArgument(response.message));
                }
                let payload = response.payload.unwrap_or(ipc::DaemonShutdownPayload {
                    daemon_ready: false,
                    runtime_disconnected: false,
                });
                println!(
                    "Daemon stop: {} (protocol v{}, ready={}, runtime_disconnected={})",
                    response.message,
                    response.protocol_version,
                    payload.daemon_ready,
                    payload.runtime_disconnected
                );
            }
            Err(err) if ipc::daemon_unreachable(&err) => {
                println!(
                    "Daemon stop: not running. Socket: {}",
                    socket_path.display()
                );
            }
            Err(err) => return Err(err),
        },
        DaemonAction::Install(args) => daemon_install::install(context, &args)?,
        DaemonAction::Uninstall(args) => daemon_install::uninstall(context, &args)?,
    }

    Ok(())
}

fn spawn_detached_daemon(context: &AppContext) -> crate::app::Result<()> {
    let current_exe = std::env::current_exe()?;
    let mut command = std::process::Command::new(current_exe);
    command
        .arg("--config")
        .arg(&context.runtime_paths.config_path)
        .arg("daemon")
        .arg("run-server")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command.spawn()?;
    Ok(())
}

async fn wait_until_daemon_ready(socket_path: &std::path::Path) -> crate::app::Result<()> {
    for _ in 0..20 {
        if ipc::ping_daemon(socket_path).await.is_ok() {
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }
    Err(crate::app::AppError::InvalidArgument(
        "daemon start failed: socket did not become reachable".to_string(),
    ))
}
