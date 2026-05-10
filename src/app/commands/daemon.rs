use crate::app::daemon::{server, supervisor};
use crate::app::runtime::AppContext;
use crate::cli::{DaemonAction, DaemonArgs};

pub async fn run(context: &AppContext, args: &DaemonArgs) -> crate::app::Result<()> {
    let socket_path = server::default_socket_path(&context.runtime_paths.runtime_dir);

    match args.action {
        DaemonAction::Start(_) => {
            println!("Starting daemon IPC listener at {}", socket_path.display());
            let (tx, rx) = supervisor::channel(32);
            let supervisor_context = context.clone();
            tokio::spawn(supervisor::run(rx, supervisor_context));
            server::serve_ping(&socket_path, tx).await?;
        }
        DaemonAction::Status(_) => match server::runtime_status_daemon(&socket_path).await {
            Ok(response) => {
                let payload = response.payload.unwrap_or(server::RuntimeStatusPayload {
                    daemon_ready: false,
                    runtime_owned: false,
                    runtime_status: "unknown".to_string(),
                    session_id: None,
                    active_config_id: None,
                    pid_running: false,
                });
                println!(
                    "Daemon status: {} (protocol v{}, ready={}, runtime_owned={}, runtime={}, session={:?}, active_config={:?}, pid_running={})",
                    response.message,
                    response.protocol_version,
                    payload.daemon_ready,
                    payload.runtime_owned,
                    payload.runtime_status,
                    payload.session_id,
                    payload.active_config_id,
                    payload.pid_running
                );
            }
            Err(err) if server::daemon_unreachable(&err) => {
                println!(
                    "Daemon status: not running (start with `xrat daemon start`). Socket: {}",
                    socket_path.display()
                );
            }
            Err(err) => return Err(err),
        },
        DaemonAction::Stop(_) => match server::daemon_shutdown_daemon(&socket_path).await {
            Ok(response) => {
                let payload = response.payload.unwrap_or(server::DaemonShutdownPayload {
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
            Err(err) if server::daemon_unreachable(&err) => {
                println!(
                    "Daemon stop: not running. Socket: {}",
                    socket_path.display()
                );
            }
            Err(err) => return Err(err),
        },
    }

    Ok(())
}
