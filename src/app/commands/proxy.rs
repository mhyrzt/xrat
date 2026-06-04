use crate::app::commands::output;
use crate::app::context::AppContext;
use crate::app::daemon::ipc;
use crate::cli::{ProxyAction, ProxyArgs};

pub async fn run(context: &AppContext, args: &ProxyArgs) -> crate::app::Result<()> {
    let socket_path = ipc::default_socket_path(&context.runtime_paths.runtime_dir);

    match &args.action {
        ProxyAction::Start(_) => {
            let response = ipc::proxy_start_daemon(&socket_path).await;
            match response {
                Ok(response) => {
                    if !response.ok {
                        return Err(crate::app::AppError::InvalidArgument(response.message));
                    }
                    println!(
                        "{}",
                        output::success(
                            format!("Proxy rotation: {}.", response.message),
                            output::color_enabled()
                        )
                    );
                    println!(
                        "{}",
                        output::notice(
                            "State is volatile and resets to config defaults on daemon restart.",
                            output::color_enabled()
                        )
                    );
                }
                Err(err) if ipc::daemon_unreachable(&err) => {
                    return Err(crate::app::AppError::InvalidArgument(format!(
                        "daemon is not running. Start it with `xrat daemon start` (socket: {})",
                        socket_path.display()
                    )));
                }
                Err(err) => return Err(err),
            }
        }
        ProxyAction::Status(status_args) => {
            let response = ipc::proxy_status_daemon(&socket_path).await;
            match response {
                Ok(response) => {
                    if !response.ok {
                        return Err(crate::app::AppError::InvalidArgument(response.message));
                    }
                    let payload = response.payload.ok_or_else(|| {
                        crate::app::AppError::InvalidArgument(
                            "proxy status response missing payload".to_string(),
                        )
                    })?;
                    if status_args.json {
                        println!("{}", serde_json::to_string_pretty(&payload)?);
                    } else {
                        println!(
                            "{}",
                            output::format_kv(
                                Some("Proxy rotation"),
                                &[
                                    (
                                        "enabled",
                                        output::bool_label(payload.rotation_enabled).to_string(),
                                    ),
                                    ("interval", format!("{}s", payload.interval_secs)),
                                    (
                                        "health trigger",
                                        output::bool_label(payload.health_trigger_enabled)
                                            .to_string(),
                                    ),
                                    ("cooldown", format!("{}s", payload.cooldown_secs)),
                                    (
                                        "cooldown active",
                                        output::bool_label(payload.cooldown_active).to_string(),
                                    ),
                                    (
                                        "active config",
                                        payload
                                            .active_config_id
                                            .map(|value| value.to_string())
                                            .unwrap_or_else(|| "-".to_string()),
                                    ),
                                    (
                                        "last trigger",
                                        payload
                                            .last_trigger
                                            .map(|trigger| match trigger {
                                                ipc::RotationTrigger::Manual => "manual",
                                                ipc::RotationTrigger::Timer => "timer",
                                                ipc::RotationTrigger::HealthCheckFailed => {
                                                    "health_check_failed"
                                                }
                                            })
                                            .unwrap_or("-")
                                            .to_string(),
                                    ),
                                    ("last result", payload.last_result),
                                    (
                                        "candidate config",
                                        payload
                                            .last_candidate_config_id
                                            .map(|value| value.to_string())
                                            .unwrap_or_else(|| "-".to_string()),
                                    ),
                                    ("candidate result", payload.last_candidate_result),
                                    (
                                        "next timer",
                                        payload
                                            .next_timer_epoch_secs
                                            .map(|value| value.to_string())
                                            .unwrap_or_else(|| "-".to_string()),
                                    ),
                                ],
                                output::color_enabled(),
                            )
                        );
                    }
                }
                Err(err) if ipc::daemon_unreachable(&err) => {
                    return Err(crate::app::AppError::InvalidArgument(format!(
                        "daemon is not running. Start it with `xrat daemon start` (socket: {})",
                        socket_path.display()
                    )));
                }
                Err(err) => return Err(err),
            }
        }
        ProxyAction::Rotate(rotate_args) => {
            let response = ipc::runtime_replace_daemon(
                &socket_path,
                ipc::RotationTrigger::Manual,
                rotate_args.config_id,
            )
            .await;
            match response {
                Ok(response) => {
                    if !response.ok {
                        return Err(crate::app::AppError::InvalidArgument(response.message));
                    }
                    let payload = response.payload.ok_or_else(|| {
                        crate::app::AppError::InvalidArgument(
                            "proxy rotate response missing payload".to_string(),
                        )
                    })?;
                    println!(
                        "{}",
                        output::success("Proxy rotation completed.", output::color_enabled())
                    );
                    println!(
                        "{}",
                        output::format_kv(
                            None,
                            &[
                                ("replaced", output::bool_label(payload.replaced).to_string()),
                                ("old session", payload.old_session_id.to_string()),
                                ("new config", payload.new_config_id.to_string()),
                                ("new session", payload.new_session_id.to_string()),
                                ("new pid", payload.new_pid.to_string()),
                            ],
                            output::color_enabled(),
                        )
                    );
                }
                Err(err) if ipc::daemon_unreachable(&err) => {
                    return Err(crate::app::AppError::InvalidArgument(format!(
                        "daemon is not running. Start it with `xrat daemon start` (socket: {})",
                        socket_path.display()
                    )));
                }
                Err(err) => return Err(err),
            }
        }
        ProxyAction::Stop(_) => {
            let response = ipc::proxy_stop_daemon(&socket_path).await;
            match response {
                Ok(response) => {
                    if !response.ok {
                        return Err(crate::app::AppError::InvalidArgument(response.message));
                    }
                    println!(
                        "{}",
                        output::success(
                            format!("Proxy rotation: {}.", response.message),
                            output::color_enabled()
                        )
                    );
                    println!(
                        "{}",
                        output::notice(
                            "State is volatile and resets to config defaults on daemon restart.",
                            output::color_enabled()
                        )
                    );
                }
                Err(err) if ipc::daemon_unreachable(&err) => {
                    return Err(crate::app::AppError::InvalidArgument(format!(
                        "daemon is not running. Start it with `xrat daemon start` (socket: {})",
                        socket_path.display()
                    )));
                }
                Err(err) => return Err(err),
            }
        }
    }

    Ok(())
}
