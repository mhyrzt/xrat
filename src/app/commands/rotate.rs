use crate::app::commands::output;
use crate::app::commands::progress::CliProgress;
use crate::app::commands::resolve::resolve_config_id;
use crate::app::context::AppContext;
use crate::app::daemon::ipc;
use crate::cli::{RotateAction, RotateArgs};
use crate::db::{ConfigListFilter, EventFilter, EventRecord};
use tokio::time::Duration;

pub async fn run(context: &AppContext, args: &RotateArgs) -> crate::app::Result<()> {
    let socket_path = ipc::default_socket_path(&context.runtime_paths.runtime_dir);

    match &args.action {
        RotateAction::Enable(_) => enable(context, &socket_path).await,
        RotateAction::Disable(_) => disable(context, &socket_path).await,
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

async fn enable(context: &AppContext, socket_path: &std::path::Path) -> crate::app::Result<()> {
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
            print_rotation_config_guidance(context, true);
            Ok(())
        }
        Err(err) if ipc::daemon_unreachable(&err) => Err(crate::app::AppError::InvalidArgument(
            daemon_unreachable_message(socket_path),
        )),
        Err(err) => Err(err),
    }
}

async fn disable(context: &AppContext, socket_path: &std::path::Path) -> crate::app::Result<()> {
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
            print_rotation_config_guidance(context, false);
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

    let stream_task = tokio::spawn(stream_rotation_events(context.clone()));
    let result =
        match ipc::runtime_replace_daemon(socket_path, ipc::RotationTrigger::Manual, config_id)
            .await
        {
            Ok(response) => {
                if !response.ok {
                    if is_no_running_runtime_session_error(&response.message) {
                        return connect_initial_runtime(context, socket_path, config_id).await;
                    }
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
                            (
                                "old session",
                                payload
                                    .old_session_id
                                    .map(|id| id.to_string())
                                    .unwrap_or_else(|| "-".to_string()),
                            ),
                            ("new config", new_config_ref),
                            ("new session", payload.new_session_id.to_string()),
                            ("new pid", payload.new_pid.to_string()),
                        ],
                        output::color_enabled(),
                    )
                );
                Ok(())
            }
            Err(err) if ipc::daemon_unreachable(&err) => Err(
                crate::app::AppError::InvalidArgument(daemon_unreachable_message(socket_path)),
            ),
            Err(err) => Err(err),
        };
    stream_task.abort();
    let _ = stream_task.await;
    result
}

async fn connect_initial_runtime(
    context: &AppContext,
    socket_path: &std::path::Path,
    config_id: Option<i64>,
) -> crate::app::Result<()> {
    let config_id = match config_id {
        Some(config_id) => config_id,
        None => initial_rotation_config_id(context).await?,
    };
    match ipc::runtime_connect_daemon(socket_path, config_id).await {
        Ok(response) => {
            if !response.ok {
                return Err(crate::app::AppError::InvalidArgument(response.message));
            }
            let payload = response.payload.ok_or_else(|| {
                crate::app::AppError::InvalidArgument(
                    "daemon connect response missing payload".to_string(),
                )
            })?;
            let new_config_ref = config_ref(context, Some(payload.config_id))
                .await?
                .unwrap_or_else(|| payload.config_id.to_string());
            println!(
                "{}",
                output::success("Proxy rotation completed.", output::color_enabled())
            );
            println!(
                "{}",
                output::format_kv(
                    None,
                    &[
                        ("replaced", output::bool_label(false).to_string()),
                        ("old session", "-".to_string()),
                        ("new config", new_config_ref),
                        ("new session", payload.session_id.to_string()),
                        ("new pid", payload.pid.to_string()),
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

async fn initial_rotation_config_id(context: &AppContext) -> crate::app::Result<i64> {
    let filter = ConfigListFilter {
        only_enabled: true,
        ..Default::default()
    };
    context
        .db
        .list_configs(&filter)
        .await?
        .into_iter()
        .map(|config| config.id)
        .min()
        .ok_or_else(|| {
            crate::app::AppError::InvalidArgument("no eligible replacement candidate".to_string())
        })
}

async fn stream_rotation_events(context: AppContext) {
    let mut cursor = latest_event_id(&context).await.unwrap_or(0);
    let filter = EventFilter {
        source: None,
        levels: None,
        limit: 0,
    };

    let mut ticker = tokio::time::interval(Duration::from_millis(200));
    let mut progress = CliProgress::disabled();
    let mut progress_active = false;
    let color = output::color_enabled();
    loop {
        ticker.tick().await;
        let events = match context.db.events_after(cursor, &filter).await {
            Ok(events) => events,
            Err(_) => continue,
        };
        for event in events {
            cursor = cursor.max(event.id);
            if event.source == crate::app::events::SOURCE_ROTATION
                && event.kind == "rotation_bulk_progress"
            {
                if let Some((done, total)) = parse_rotation_progress_detail(event.detail.as_deref())
                {
                    if !progress_active && total > 0 {
                        progress = CliProgress::bar(
                            true,
                            total as u64,
                            format!("rotation candidate tests {done}/{total}"),
                        );
                        progress_active = true;
                    }
                    progress.set_position(done as u64);
                    progress.set_message(format!("rotation candidate tests {done}/{total}"));
                    if done >= total && total > 0 {
                        progress.finish_with_message(format!(
                            "rotation candidate tests {done}/{total}"
                        ));
                        println!(
                            "{}",
                            output::notice(
                                format!("Rotation candidate testing finished: {done}/{total}."),
                                color
                            )
                        );
                        progress = CliProgress::disabled();
                        progress_active = false;
                    }
                }
                continue;
            }

            if should_render_rotation_event(&event) {
                println!(
                    "{}",
                    output::notice(
                        format!(
                            "[{}:{}] {}",
                            event.source,
                            event.kind,
                            event.message.replace(['\n', '\r', '\t'], " ")
                        ),
                        output::color_enabled()
                    )
                );
            }
        }
    }
}

async fn latest_event_id(context: &AppContext) -> crate::app::Result<i64> {
    let events = context
        .db
        .list_events(&EventFilter {
            source: None,
            levels: None,
            limit: 1,
        })
        .await?;
    Ok(events.first().map(|event| event.id).unwrap_or(0))
}

fn should_render_rotation_event(event: &EventRecord) -> bool {
    if event.source == crate::app::events::SOURCE_ROTATION {
        return true;
    }
    if event.source == crate::app::events::SOURCE_SUBSCRIPTION {
        return event.kind.contains("refresh");
    }
    if event.source == crate::app::events::SOURCE_TEST {
        return event.kind == "test_run";
    }
    false
}

fn parse_rotation_progress_detail(detail: Option<&str>) -> Option<(usize, usize)> {
    let detail = detail?;
    let parsed: serde_json::Value = serde_json::from_str(detail).ok()?;
    let done = parsed.get("done")?.as_u64()? as usize;
    let total = parsed.get("total")?.as_u64()? as usize;
    Some((done, total))
}

fn print_rotation_config_guidance(context: &AppContext, enabled: bool) {
    let (config_line, instruction_line) =
        rotation_config_guidance_lines(&context.runtime_paths.config_path, enabled);
    println!("{}", output::notice(config_line, output::color_enabled()));
    println!(
        "{}",
        output::notice(instruction_line, output::color_enabled())
    );
}

fn rotation_config_guidance_lines(
    config_path: &std::path::Path,
    enabled: bool,
) -> (String, String) {
    let persisted_value = if enabled { "true" } else { "false" };
    let config_line = format!("Config file: {}", config_path.display());
    let instruction_line = format!(
        "To make this permanent, set [runtime.rotation].enabled = {persisted_value} in that file."
    );
    (config_line, instruction_line)
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
    message
}

fn is_no_running_runtime_session_error(message: &str) -> bool {
    message.contains("no running runtime session to replace")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::NewEvent;

    #[test]
    fn daemon_unreachable_message_mentions_persistent_install() {
        let message = daemon_unreachable_message(std::path::Path::new("/tmp/xrat.sock"));
        assert!(message.contains("xrat daemon start"));
        assert!(message.contains("xrat daemon install --start"));
        assert!(message.contains("/tmp/xrat.sock"));
    }

    #[test]
    fn no_running_runtime_session_error_is_detected_for_fallback() {
        assert!(is_no_running_runtime_session_error(
            "no running runtime session to replace"
        ));
        assert!(!is_no_running_runtime_session_error(
            "no eligible replacement candidate"
        ));
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

    #[test]
    fn parse_rotation_progress_detail_reads_json_fields() {
        assert_eq!(
            parse_rotation_progress_detail(Some(r#"{"done":3,"total":9}"#)),
            Some((3, 9))
        );
    }

    #[test]
    fn parse_rotation_progress_detail_rejects_missing_fields() {
        assert_eq!(parse_rotation_progress_detail(Some(r#"{"done":3}"#)), None);
    }

    #[test]
    fn parse_rotation_progress_detail_rejects_invalid_json() {
        assert_eq!(parse_rotation_progress_detail(Some("not-json")), None);
    }

    #[test]
    fn should_render_rotation_event_filters_expected_sources() {
        let base = NewEvent {
            level: "info".to_string(),
            source: crate::app::events::SOURCE_ROTATION.to_string(),
            kind: "proxy_rotated".to_string(),
            config_id: None,
            session_id: None,
            message: "ok".to_string(),
            detail: None,
        };
        let rotation = EventRecord {
            id: 1,
            level: base.level.clone(),
            source: base.source.clone(),
            kind: base.kind.clone(),
            config_id: base.config_id,
            session_id: base.session_id,
            message: base.message.clone(),
            detail: None,
            created_at: "now".to_string(),
        };
        assert!(should_render_rotation_event(&rotation));

        let subscription = EventRecord {
            source: crate::app::events::SOURCE_SUBSCRIPTION.to_string(),
            kind: "subscription_refresh_result".to_string(),
            ..rotation.clone()
        };
        assert!(should_render_rotation_event(&subscription));

        let test_run = EventRecord {
            source: crate::app::events::SOURCE_TEST.to_string(),
            kind: "test_run".to_string(),
            ..rotation.clone()
        };
        assert!(should_render_rotation_event(&test_run));

        let unrelated = EventRecord {
            source: crate::app::events::SOURCE_DAEMON.to_string(),
            kind: "daemon_started".to_string(),
            ..rotation
        };
        assert!(!should_render_rotation_event(&unrelated));
    }

    #[test]
    fn rotation_config_guidance_lines_include_config_key_and_value() {
        let (config_line, instruction_line) =
            rotation_config_guidance_lines(std::path::Path::new("/tmp/config.toml"), true);
        assert!(config_line.contains("Config file: "));
        assert!(config_line.contains("/tmp/config.toml"));
        assert!(instruction_line.contains("[runtime.rotation].enabled = true"));
    }
}
