use crate::app::commands::output;
use crate::app::daemon::ipc::{DaemonResponse, RuntimeStatusPayload};

pub(super) fn print_daemon_status(
    response: DaemonResponse<RuntimeStatusPayload>,
    as_json: bool,
) -> crate::app::Result<()> {
    if !response.ok {
        return Err(crate::app::AppError::InvalidArgument(response.message));
    }
    let payload = response.payload.ok_or_else(|| {
        crate::app::AppError::InvalidArgument("daemon status response missing payload".to_string())
    })?;

    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "daemon": true,
                "runtime": payload.runtime_status,
                "runtime_owned": payload.runtime_owned,
                "session_id": payload.session_id,
                "active_config_id": payload.active_config_id,
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
