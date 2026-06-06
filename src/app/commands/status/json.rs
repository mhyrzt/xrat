use crate::app::runtime_service::{RuntimeEndpointHealth, RuntimeStatusSnapshot};

pub(super) fn print_json_status(snapshot: &RuntimeStatusSnapshot) -> crate::app::Result<()> {
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
        "ref": &config.r#ref,
        "name": &config.name,
        "protocol": &config.protocol,
        "address": &config.address,
        "port": config.port,
        "is_enabled": config.is_enabled,
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
