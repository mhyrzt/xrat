use crate::app::daemon::server;
use crate::app::runtime::AppContext;
use crate::app::runtime_service::{RuntimeEndpointHealth, RuntimeService};
use crate::cli::StatusArgs;

pub async fn run(context: &AppContext, args: &StatusArgs) -> crate::app::Result<()> {
    let socket_path = server::default_socket_path(&context.runtime_paths.runtime_dir);
    match server::runtime_status_daemon(&socket_path).await {
        Ok(response) => {
            if !response.ok {
                return Err(crate::app::AppError::InvalidArgument(response.message));
            }
            let payload = response.payload.ok_or_else(|| {
                crate::app::AppError::InvalidArgument(
                    "daemon status response missing payload".to_string(),
                )
            })?;
            if args.json {
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
            } else {
                println!("Runtime: {} (daemon)", payload.runtime_status);
                println!("Owned: {}", payload.runtime_owned);
                if let Some(session_id) = payload.session_id {
                    println!("Session: {session_id}");
                }
                if let Some(config_id) = payload.active_config_id {
                    println!("Active config: {config_id}");
                }
                println!("PID running: {}", payload.pid_running);
            }
            return Ok(());
        }
        Err(err) if server::daemon_unreachable(&err) => {
            tracing::info!("daemon not reachable; using direct runtime status path");
        }
        Err(err) => return Err(err),
    }

    let snapshot = RuntimeService::new(context).status().await?;

    if args.json {
        print_json_status(&snapshot)?;
        return Ok(());
    }

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

fn print_json_status(
    snapshot: &crate::app::runtime_service::RuntimeStatusSnapshot,
) -> crate::app::Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "runtime": snapshot.status.as_str(),
            "session": snapshot.session.as_ref().map(|session| serde_json::json!({
                "id": session.id,
                "config_id": session.config_id,
                "config_missing": session.config_id.is_some() && snapshot.session_config.is_none(),
                "status": session.status.as_str(),
                "process_id": session.process_id,
                "pid_running": snapshot.pid_running,
                "failure_reason": &session.failure_reason,
                "started_at": &session.started_at,
                "stopped_at": &session.stopped_at,
                "created_at": &session.created_at,
                "updated_at": &session.updated_at,
            })),
            "session_config": snapshot.session_config.as_ref().map(config_json),
            "active_config": snapshot.active_config.as_ref().map(config_json),
            "selected_config": snapshot.selected_config.as_ref().map(config_json),
            "inbounds": {
                "socks": health_json(snapshot.inbound_health.socks.as_ref()),
                "http": health_json(snapshot.inbound_health.http.as_ref()),
                "shadowsocks": health_json(snapshot.inbound_health.shadowsocks.as_ref()),
            },
            "database": snapshot.database_label,
        }))?
    );
    Ok(())
}

fn config_json(config: &crate::db::ConfigRecord) -> serde_json::Value {
    serde_json::json!({
        "id": config.id,
        "name": &config.name,
        "protocol": &config.protocol,
        "address": &config.address,
        "port": config.port,
        "is_enabled": config.is_enabled,
        "is_selected": config.is_selected,
        "is_active": config.is_active,
    })
}

fn health_json(health: Option<&RuntimeEndpointHealth>) -> serde_json::Value {
    health
        .map(|health| {
            serde_json::json!({
                "host": &health.endpoint.host,
                "port": health.endpoint.port,
                "state": health.state.as_str(),
            })
        })
        .unwrap_or(serde_json::Value::Null)
}

fn print_inbound(label: &str, health: Option<&RuntimeEndpointHealth>) {
    if let Some(health) = health {
        let endpoint = super::runtime_output::format_inbound_endpoint(
            &health.endpoint.host,
            health.endpoint.port,
        );
        println!("{label}: {endpoint} ({})", health.state.as_str());
    }
}
