use crate::app::daemon::server::DaemonResponse;
use crate::app::daemon::server::RuntimeStatusPayload;

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
            }))?
        );
        return Ok(());
    }

    println!("Runtime: {} (daemon)", payload.runtime_status);
    println!("Owned: {}", payload.runtime_owned);
    if let Some(session_id) = payload.session_id {
        println!("Session: {session_id}");
    }
    if let Some(config_id) = payload.active_config_id {
        println!("Active config: {config_id}");
    }
    println!("PID running: {}", payload.pid_running);
    Ok(())
}
