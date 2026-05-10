use crate::app::daemon::server;
use crate::app::runtime::AppContext;
use crate::app::runtime_service::{ConnectRequest, RuntimeEndpoint, RuntimeService};
use crate::cli::ConnectArgs;

pub async fn run(context: &AppContext, args: &ConnectArgs) -> crate::app::Result<()> {
    let socket_path = server::default_socket_path(&context.runtime_paths.runtime_dir);
    match server::runtime_connect_daemon(&socket_path, args.id).await {
        Ok(response) => {
            if !response.ok {
                return Err(crate::app::AppError::InvalidArgument(response.message));
            }
            let payload = response.payload.ok_or_else(|| {
                crate::app::AppError::InvalidArgument(
                    "daemon connect response missing payload".to_string(),
                )
            })?;
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "status": "connected",
                        "daemon": true,
                        "config": { "id": payload.config_id },
                        "session": { "id": payload.session_id, "pid": payload.pid },
                    }))?
                );
            } else {
                println!("Connected config {} via daemon", payload.config_id);
                println!("Session: {}", payload.session_id);
                println!("PID: {}", payload.pid);
            }
            return Ok(());
        }
        Err(err) if server::daemon_unreachable(&err) => {
            tracing::info!("daemon not reachable; using direct runtime connect path");
        }
        Err(err) => return Err(err),
    }

    let result = RuntimeService::new(context)
        .connect(ConnectRequest { config_id: args.id })
        .await?;

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "status": "connected",
                "config": {
                    "id": result.config.id,
                    "name": &result.config.name,
                    "protocol": &result.config.protocol,
                    "address": &result.config.address,
                    "port": result.config.port,
                },
                "session": {
                    "id": result.session_id,
                    "pid": result.pid,
                    "runtime_config": &result.runtime_config_path,
                },
                "inbounds": {
                    "socks": endpoint_json(result.endpoints.socks.as_ref()),
                    "http": endpoint_json(result.endpoints.http.as_ref()),
                    "shadowsocks": endpoint_json(result.endpoints.shadowsocks.as_ref()),
                }
            }))?
        );
        return Ok(());
    }

    println!("Connected config {}", result.config.id);
    if let Some(name) = &result.config.name {
        println!("Name: {name}");
    }
    if let Some(inbound) = &result.endpoints.socks {
        println!(
            "SOCKS: {}",
            super::runtime_output::format_inbound_endpoint(&inbound.host, inbound.port)
        );
    }
    if let Some(inbound) = &result.endpoints.http {
        println!(
            "HTTP: {}",
            super::runtime_output::format_inbound_endpoint(&inbound.host, inbound.port)
        );
    }
    if let Some(inbound) = &result.endpoints.shadowsocks {
        println!(
            "Shadowsocks: {}",
            super::runtime_output::format_inbound_endpoint(&inbound.host, inbound.port)
        );
    }
    println!("PID: {}", result.pid);
    println!("Runtime config: {}", result.runtime_config_path.display());

    Ok(())
}

fn endpoint_json(endpoint: Option<&RuntimeEndpoint>) -> serde_json::Value {
    endpoint
        .map(|endpoint| {
            serde_json::json!({
                "host": &endpoint.host,
                "port": endpoint.port,
            })
        })
        .unwrap_or(serde_json::Value::Null)
}
