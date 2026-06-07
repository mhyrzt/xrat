use crate::app::commands::output;
use crate::app::commands::resolve::resolve_config_id;
use crate::app::context::AppContext;
use crate::app::daemon::ipc;
use crate::cli::{RotateAction, RotateArgs};

pub async fn run(context: &AppContext, args: &RotateArgs) -> crate::app::Result<()> {
    let socket_path = ipc::default_socket_path(&context.runtime_paths.runtime_dir);

    match &args.action {
        RotateAction::Start(_) => start(&socket_path).await,
        RotateAction::Stop(_) => stop(&socket_path).await,
        RotateAction::Status(status_args) => status(context, &socket_path, status_args.json).await,
        RotateAction::Now(now_args) => {
            now(
                context,
                &socket_path,
                now_args.config_id.as_deref(),
                now_args.refresh,
            )
            .await
        }
    }
}

async fn start(socket_path: &std::path::Path) -> crate::app::Result<()> {
    match ipc::proxy_start_daemon(socket_path).await {
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
            Ok(())
        }
        Err(err) if ipc::daemon_unreachable(&err) => Err(crate::app::AppError::InvalidArgument(
            daemon_unreachable_message(socket_path),
        )),
        Err(err) => Err(err),
    }
}

async fn stop(socket_path: &std::path::Path) -> crate::app::Result<()> {
    match ipc::proxy_stop_daemon(socket_path).await {
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
            Ok(())
        }
        Err(err) if ipc::daemon_unreachable(&err) => Err(crate::app::AppError::InvalidArgument(
            daemon_unreachable_message(socket_path),
        )),
        Err(err) => Err(err),
    }
}

async fn status(
    context: &AppContext,
    socket_path: &std::path::Path,
    json: bool,
) -> crate::app::Result<()> {
    match ipc::proxy_status_daemon(socket_path).await {
        Ok(response) => {
            if !response.ok {
                return Err(crate::app::AppError::InvalidArgument(response.message));
            }
            let payload = response.payload.ok_or_else(|| {
                crate::app::AppError::InvalidArgument(
                    "proxy status response missing payload".to_string(),
                )
            })?;
            let active_config_ref = config_ref(context, payload.active_config_id).await?;
            let last_candidate_config_ref =
                config_ref(context, payload.last_candidate_config_id).await?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "rotation_enabled": payload.rotation_enabled,
                        "interval_secs": payload.interval_secs,
                        "health_trigger_enabled": payload.health_trigger_enabled,
                        "cooldown_secs": payload.cooldown_secs,
                        "cooldown_active": payload.cooldown_active,
                        "active_config_ref": active_config_ref,
                        "last_trigger": payload.last_trigger,
                        "last_result": payload.last_result,
                        "last_candidate_config_ref": last_candidate_config_ref,
                        "last_candidate_result": payload.last_candidate_result,
                        "next_timer_epoch_secs": payload.next_timer_epoch_secs,
                    }))?
                );
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
                                output::bool_label(payload.health_trigger_enabled).to_string(),
                            ),
                            ("cooldown", format!("{}s", payload.cooldown_secs)),
                            (
                                "cooldown active",
                                output::bool_label(payload.cooldown_active).to_string(),
                            ),
                            (
                                "active config",
                                active_config_ref.unwrap_or_else(|| "-".to_string()),
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
                            ("last result", friendly_result(&payload.last_result)),
                            (
                                "candidate config",
                                last_candidate_config_ref.unwrap_or_else(|| "-".to_string()),
                            ),
                            (
                                "candidate result",
                                friendly_result(&payload.last_candidate_result),
                            ),
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
            Ok(())
        }
        Err(err) if ipc::daemon_unreachable(&err) => Err(crate::app::AppError::InvalidArgument(
            daemon_unreachable_message(socket_path),
        )),
        Err(err) => Err(err),
    }
}

async fn now(
    context: &AppContext,
    socket_path: &std::path::Path,
    config_id: Option<&str>,
    refresh: bool,
) -> crate::app::Result<()> {
    if refresh {
        let outcome = crate::app::subscription_refresh::refresh_all(context).await;
        println!(
            "{}",
            output::format_kv(
                Some("Refreshed subscriptions"),
                &[
                    ("attempted", outcome.attempted.to_string()),
                    ("succeeded", outcome.succeeded.to_string()),
                    ("failed", outcome.failed.to_string()),
                    ("removed configs", outcome.removed_configs.to_string()),
                ],
                output::color_enabled(),
            )
        );
    }

    let config_id = match config_id {
        Some(raw) => Some(resolve_config_id(context, raw).await?),
        None => None,
    };

    match ipc::runtime_replace_daemon(socket_path, ipc::RotationTrigger::Manual, config_id).await {
        Ok(response) => {
            if !response.ok {
                return Err(crate::app::AppError::InvalidArgument(rotate_error_message(
                    response.message,
                )));
            }
            let payload = response.payload.ok_or_else(|| {
                crate::app::AppError::InvalidArgument(
                    "proxy rotate response missing payload".to_string(),
                )
            })?;
            let new_config_ref = config_ref(context, Some(payload.new_config_id))
                .await?
                .unwrap_or_else(|| "-".to_string());
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
                        ("new config", new_config_ref),
                        ("new session", payload.new_session_id.to_string()),
                        ("new pid", payload.new_pid.to_string()),
                    ],
                    output::color_enabled(),
                )
            );
            Ok(())
        }
        Err(err) if ipc::daemon_unreachable(&err) => Err(crate::app::AppError::InvalidArgument(
            daemon_unreachable_message(socket_path),
        )),
        Err(err) => Err(err),
    }
}

fn daemon_unreachable_message(socket_path: &std::path::Path) -> String {
    format!(
        "daemon is not running. Start it now with `xrat daemon start`, or install it for login startup with `xrat daemon install --start` (socket: {})",
        socket_path.display()
    )
}

async fn config_ref(
    context: &AppContext,
    config_id: Option<i64>,
) -> crate::app::Result<Option<String>> {
    let Some(config_id) = config_id else {
        return Ok(None);
    };
    Ok(context
        .db
        .get_config_by_id(config_id)
        .await?
        .map(|config| config.r#ref))
}

/// Translate internal scheduler sentinel values into human-readable status text.
/// JSON output keeps the raw sentinels for stable machine parsing.
fn friendly_result(result: &str) -> String {
    match result {
        "never_triggered" => "auto-rotation has not run yet".to_string(),
        "never_selected" => "no candidate selected yet".to_string(),
        other => other.to_string(),
    }
}

fn rotate_error_message(message: String) -> String {
    if message.contains("no running runtime session to replace") {
        return format!("{message}. Start a runtime first with `xrat connect <id>`");
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_unreachable_message_mentions_persistent_install() {
        let message = daemon_unreachable_message(std::path::Path::new("/tmp/xrat.sock"));
        assert!(message.contains("xrat daemon start"));
        assert!(message.contains("xrat daemon install --start"));
        assert!(message.contains("/tmp/xrat.sock"));
    }

    #[test]
    fn rotate_error_message_mentions_connect_when_no_runtime_is_running() {
        let message = rotate_error_message("no running runtime session to replace".to_string());
        assert!(message.contains("xrat connect <id>"));
    }

    #[test]
    fn friendly_result_translates_sentinels() {
        assert_eq!(
            friendly_result("never_triggered"),
            "auto-rotation has not run yet"
        );
        assert_eq!(
            friendly_result("never_selected"),
            "no candidate selected yet"
        );
        assert_eq!(friendly_result("replaced"), "replaced");
    }
}
