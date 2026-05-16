use crate::app::context::AppContext;
use crate::app::daemon::server;
use crate::cli::{ProxyAction, ProxyArgs};

pub async fn run(context: &AppContext, args: &ProxyArgs) -> crate::app::Result<()> {
    let socket_path = server::default_socket_path(&context.runtime_paths.runtime_dir);

    match &args.action {
        ProxyAction::Start(_) => {
            let response = server::proxy_start_daemon(&socket_path).await;
            match response {
                Ok(response) => {
                    if !response.ok {
                        return Err(crate::app::AppError::InvalidArgument(response.message));
                    }
                    println!("Proxy rotation: {}", response.message);
                }
                Err(err) if server::daemon_unreachable(&err) => {
                    return Err(crate::app::AppError::InvalidArgument(format!(
                        "daemon is not running. Start it with `xrat daemon start` (socket: {})",
                        socket_path.display()
                    )));
                }
                Err(err) => return Err(err),
            }
        }
        ProxyAction::Status(status_args) => {
            let response = server::proxy_status_daemon(&socket_path).await;
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
                            "Proxy rotation: enabled={}, interval_secs={}, health_trigger_enabled={}, cooldown_secs={}, cooldown_active={}, active_config={:?}, last_trigger={:?}, last_result={}, last_candidate_config={:?}, last_candidate_result={}, next_timer_at={:?}",
                            payload.rotation_enabled,
                            payload.interval_secs,
                            payload.health_trigger_enabled,
                            payload.cooldown_secs,
                            payload.cooldown_active,
                            payload.active_config_id,
                            payload.last_trigger,
                            payload.last_result,
                            payload.last_candidate_config_id,
                            payload.last_candidate_result,
                            payload.next_timer_epoch_secs
                        );
                    }
                }
                Err(err) if server::daemon_unreachable(&err) => {
                    return Err(crate::app::AppError::InvalidArgument(format!(
                        "daemon is not running. Start it with `xrat daemon start` (socket: {})",
                        socket_path.display()
                    )));
                }
                Err(err) => return Err(err),
            }
        }
        ProxyAction::Rotate(rotate_args) => {
            let response = server::runtime_replace_daemon(
                &socket_path,
                server::RotationTrigger::Manual,
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
                        "Proxy rotate: replaced={}, old_session_id={}, new_config_id={}, new_session_id={}, new_pid={}",
                        payload.replaced,
                        payload.old_session_id,
                        payload.new_config_id,
                        payload.new_session_id,
                        payload.new_pid
                    );
                }
                Err(err) if server::daemon_unreachable(&err) => {
                    return Err(crate::app::AppError::InvalidArgument(format!(
                        "daemon is not running. Start it with `xrat daemon start` (socket: {})",
                        socket_path.display()
                    )));
                }
                Err(err) => return Err(err),
            }
        }
        ProxyAction::Stop(_) => {
            let response = server::proxy_stop_daemon(&socket_path).await;
            match response {
                Ok(response) => {
                    if !response.ok {
                        return Err(crate::app::AppError::InvalidArgument(response.message));
                    }
                    println!("Proxy rotation: {}", response.message);
                }
                Err(err) if server::daemon_unreachable(&err) => {
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
