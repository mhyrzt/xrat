use crate::app::daemon::server::DaemonResponse;
use crate::app::daemon::server::RuntimeStatusPayload;
use crate::app::runtime_service::{RuntimeEndpointHealth, RuntimeStatusSnapshot};

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

pub(super) fn print_direct_status(snapshot: RuntimeStatusSnapshot) -> crate::app::Result<()> {
    let Some(session) = snapshot.session else {
        println!("Runtime: {}", snapshot.status.as_str());
        if let Some(selected) = snapshot.selected_config {
            println!("Selected config: {}", selected.id);
        }
        println!("Database: {}", snapshot.database_label);
        return Ok(());
    };

    println!("Runtime: {}", snapshot.status.as_str());
    println!("Session: {}", session.id);
    if let Some(config_id) = session.config_id {
        if snapshot.session_config.is_some() {
            println!("Session config: {config_id}");
        } else {
            println!("Session config: {config_id} (missing/deleted)");
        }
    }
    if let Some(active) = snapshot.active_config {
        println!("Active config: {}", active.id);
    }
    if let Some(selected) = snapshot.selected_config {
        println!("Selected config: {}", selected.id);
    }
    print_inbound("SOCKS", snapshot.inbound_health.socks.as_ref());
    print_inbound("HTTP", snapshot.inbound_health.http.as_ref());
    print_inbound("Shadowsocks", snapshot.inbound_health.shadowsocks.as_ref());
    if let Some(pid) = session.process_id {
        println!(
            "PID: {pid} ({})",
            if snapshot.pid_running {
                "running"
            } else {
                "not running"
            }
        );
    }
    if let Some(started_at) = session.started_at {
        println!("Started: {started_at}");
    }
    if let Some(stopped_at) = session.stopped_at {
        println!("Stopped: {stopped_at}");
    }
    if let Some(reason) = session.failure_reason {
        println!("Failure: {reason}");
    }
    println!("Database: {}", snapshot.database_label);
    Ok(())
}

fn print_inbound(label: &str, health: Option<&RuntimeEndpointHealth>) {
    if let Some(health) = health {
        let endpoint = super::super::runtime_output::format_inbound_endpoint(
            &health.endpoint.host,
            health.endpoint.port,
        );
        println!("{label}: {endpoint} ({})", health.state.as_str());
    }
}
