use crate::app::commands::output;
use crate::app::context::AppContext;
use crate::app::daemon::ipc::{DaemonResponse, RuntimeStatusPayload};

pub(super) async fn print_daemon_status(
    context: &AppContext,
    response: DaemonResponse<RuntimeStatusPayload>,
    as_json: bool,
) -> crate::app::Result<()> {
    if !response.ok {
        return Err(crate::app::AppError::InvalidArgument(response.message));
    }
    let payload = response.payload.ok_or_else(|| {
        crate::app::AppError::InvalidArgument("daemon status response missing payload".to_string())
    })?;
    let active_config_ref = config_ref(context, payload.active_config_id).await?;

    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "daemon": true,
                "runtime": payload.runtime_status,
                "runtime_owned": payload.runtime_owned,
                "session_id": payload.session_id,
                "active_config_ref": active_config_ref,
                "pid_running": payload.pid_running,
                "http_api_enabled": payload.http_api_enabled,
                "http_api_addr": payload.http_api_addr,
            }))?
        );
        return Ok(());
    }

    println!(
        "{}",
        output::format_kv(
            Some("Runtime"),
            &[
                ("mode", "daemon".to_string()),
                ("status", payload.runtime_status),
                (
                    "owned",
                    output::bool_label(payload.runtime_owned).to_string()
                ),
                (
                    "session",
                    payload
                        .session_id
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                ),
                (
                    "active config",
                    active_config_ref.unwrap_or_else(|| "-".to_string()),
                ),
                (
                    "pid running",
                    output::bool_label(payload.pid_running).to_string(),
                ),
                (
                    "http api",
                    if payload.http_api_enabled {
                        format!(
                            "enabled ({})",
                            payload.http_api_addr.as_deref().unwrap_or("unknown")
                        )
                    } else {
                        "disabled".to_string()
                    },
                ),
            ],
            output::color_enabled(),
        )
    );
    Ok(())
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
