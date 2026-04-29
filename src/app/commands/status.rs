use crate::app::runtime::AppContext;
use crate::cli::StatusArgs;
use crate::db::RuntimeSessionStatus;
use crate::xray::runtime as xray_runtime;

pub async fn run(context: &AppContext, _args: &StatusArgs) -> crate::app::Result<()> {
    let active_state = super::runtime_lifecycle::active_session_state(context).await?;
    let latest = context.db.get_latest_runtime_session().await?;
    let active = context.db.get_active_config().await?;
    let selected = context.db.get_selected_config().await?;

    let Some(session) = latest else {
        println!("Runtime: stopped");
        if let Some(selected) = selected {
            println!("Selected config: {}", selected.id);
        }
        println!("Database: {}", context.runtime_paths.database_label);
        return Ok(());
    };

    let pid_running = session
        .process_id
        .map(xray_runtime::process_is_running)
        .unwrap_or(false);
    let status = if matches!(
        active_state,
        super::runtime_lifecycle::ActiveSessionState::Stale(_)
    ) {
        "stale reconciled"
    } else if matches!(session.status, RuntimeSessionStatus::Running) && !pid_running {
        "stale"
    } else {
        session.status.as_str()
    };

    println!("Runtime: {status}");
    println!("Session: {}", session.id);
    if let Some(config_id) = session.config_id {
        println!("Session config: {config_id}");
    }
    if let Some(active) = active {
        println!("Active config: {}", active.id);
    }
    if let Some(selected) = selected {
        println!("Selected config: {}", selected.id);
    }
    if let Some(port) = session.mixed_port {
        println!("Local port: {port}");
    }
    if let Some(pid) = session.process_id {
        println!(
            "PID: {pid} ({})",
            if pid_running {
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
    println!("Database: {}", context.runtime_paths.database_label);

    Ok(())
}
