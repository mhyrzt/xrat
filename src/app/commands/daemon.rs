use crate::app::commands::daemon_install;
use crate::app::commands::output;
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
                println!(
                    "{}",
                    output::notice("Daemon already running.", output::color_enabled())
                );
                println!(
                    "{}",
                    output::format_kv(
                        None,
                        &[("socket", socket_path.display().to_string())],
                        output::color_enabled()
                    )
                );
                return Ok(());
            }
            spawn_detached_daemon(context)?;
            wait_until_daemon_ready(&socket_path).await?;
            println!(
                "{}",
                output::success("Daemon started.", output::color_enabled())
            );
            println!(
                "{}",
                output::format_kv(
                    None,
                    &[("socket", socket_path.display().to_string())],
                    output::color_enabled()
                )
            );
        }
        DaemonAction::Restart(_) => {
            let was_running = ipc::ping_daemon(&socket_path).await.is_ok();
            if was_running {
                match ipc::daemon_shutdown_daemon(&socket_path).await {
                    Ok(_) => {}
                    Err(err) if ipc::daemon_unreachable(&err) => {}
                    Err(err) => return Err(err),
                }
                wait_until_daemon_stopped(&socket_path).await?;
            }
            spawn_detached_daemon(context)?;
            wait_until_daemon_ready(&socket_path).await?;
            let message = if was_running {
                "Daemon restarted."
            } else {
                "Daemon was not running; started fresh."
            };
            println!("{}", output::success(message, output::color_enabled()));
            println!(
                "{}",
                output::format_kv(
                    None,
                    &[("socket", socket_path.display().to_string())],
                    output::color_enabled()
                )
            );
        }
        DaemonAction::RunServer(_) => {
            let (tx, rx) = supervisor::channel(32);
            let supervisor_context = context.clone();
            tokio::spawn(supervisor::run(rx, supervisor_context));

            crate::app::events::record(
                &context.db,
                crate::app::events::LEVEL_INFO,
                crate::app::events::SOURCE_DAEMON,
                "daemon_started",
                "Daemon supervisor started",
                None,
                None,
                None,
            )
            .await;

            let http_handle = if context.app_config.server.enabled {
                let server_settings = context.app_config.server.clone();
                let routing_settings = context.app_config.routing.clone();
                let db = context.db.clone();
                let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
                let handle = tokio::spawn(async move {
                    if let Err(err) = crate::server::serve_with_shutdown(
                        db,
                        &server_settings,
                        &routing_settings,
                        shutdown_rx,
                    )
                    .await
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
                    "{}",
                    output::format_kv(
                        Some("Daemon"),
                        &[
                            ("status", response.message),
                            ("protocol", format!("v{}", response.protocol_version)),
                            (
                                "ready",
                                output::bool_label(payload.daemon_ready).to_string()
                            ),
                            (
                                "runtime owned",
                                output::bool_label(payload.runtime_owned).to_string(),
                            ),
                            ("runtime", payload.runtime_status),
                            (
                                "session",
                                payload
                                    .session_id
                                    .map(|value| value.to_string())
                                    .unwrap_or_else(|| "-".to_string()),
                            ),
                            (
                                "active config",
                                payload
                                    .active_config_id
                                    .map(|value| value.to_string())
                                    .unwrap_or_else(|| "-".to_string()),
                            ),
                            (
                                "pid running",
                                output::bool_label(payload.pid_running).to_string(),
                            ),
                            (
                                "http api",
                                output::bool_label(payload.http_api_enabled).to_string(),
                            ),
                            ("http addr", output::dash(payload.http_api_addr.as_deref()),),
                        ],
                        output::color_enabled(),
                    )
                );
            }
            Err(err) if ipc::daemon_unreachable(&err) => {
                println!(
                    "{}",
                    output::warn(
                        "Daemon is not running. Start with `xrat daemon start`.",
                        output::color_enabled()
                    )
                );
                println!(
                    "{}",
                    output::format_kv(
                        None,
                        &[("socket", socket_path.display().to_string())],
                        output::color_enabled()
                    )
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
                    "{}",
                    output::success(response.message, output::color_enabled())
                );
                println!(
                    "{}",
                    output::format_kv(
                        None,
                        &[
                            ("protocol", format!("v{}", response.protocol_version)),
                            (
                                "ready",
                                output::bool_label(payload.daemon_ready).to_string()
                            ),
                            (
                                "runtime disconnected",
                                output::bool_label(payload.runtime_disconnected).to_string(),
                            ),
                        ],
                        output::color_enabled(),
                    )
                );
            }
            Err(err) if ipc::daemon_unreachable(&err) => {
                println!(
                    "{}",
                    output::notice("Daemon is not running.", output::color_enabled())
                );
                println!(
                    "{}",
                    output::format_kv(
                        None,
                        &[("socket", socket_path.display().to_string())],
                        output::color_enabled()
                    )
                );
            }
            Err(err) => return Err(err),
        },
        DaemonAction::Install(args) => daemon_install::install(context, args)?,
        DaemonAction::Uninstall(args) => daemon_install::uninstall(context, args)?,
    }

    Ok(())
}

fn spawn_detached_daemon(context: &AppContext) -> crate::app::Result<()> {
    let current_exe = std::env::current_exe()?;
    let runtime_dir = &context.runtime_paths.runtime_dir;
    std::fs::create_dir_all(runtime_dir)?;
    let log_path = runtime_dir.join("daemon.log");
    let stdout_log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    let stderr_log = stdout_log.try_clone()?;

    let mut command = std::process::Command::new(current_exe);
    command
        .arg("--config")
        .arg(&context.runtime_paths.config_path)
        .arg("daemon")
        .arg("run-server")
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_log))
        .stderr(Stdio::from(stderr_log));
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

async fn wait_until_daemon_stopped(socket_path: &std::path::Path) -> crate::app::Result<()> {
    for _ in 0..50 {
        if ipc::ping_daemon(socket_path).await.is_err() {
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }
    Err(crate::app::AppError::InvalidArgument(
        "daemon restart failed: previous daemon did not stop in time".to_string(),
    ))
}
