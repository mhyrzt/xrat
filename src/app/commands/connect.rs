use std::time::Duration;

use crate::app::runtime::AppContext;
use crate::app::{AppError, config::defaults};
use crate::cli::ConnectArgs;
use crate::db::{ConfigRecord, RuntimeSessionInsert, RuntimeSessionStatus};
use crate::model::{Node, Protocol};
use crate::xray::config::Inbound;
use crate::xray::{generate_runtime_config_for_inbounds, runtime as xray_runtime};

pub async fn run(context: &AppContext, args: &ConnectArgs) -> crate::app::Result<()> {
    let Some(config) = context.db.get_config_by_id(args.id).await? else {
        return Err(AppError::InvalidArgument(format!(
            "config {} was not found",
            args.id
        )));
    };
    if !config.is_enabled {
        return Err(AppError::InvalidArgument(format!(
            "config {} is disabled",
            args.id
        )));
    }

    match super::runtime_lifecycle::active_session_state(context).await? {
        super::runtime_lifecycle::ActiveSessionState::Running(session) => {
            if !context.app_config.runtime.replace_active_session {
                tracing::warn!(
                    session_id = session.id,
                    "active runtime session blocks connect"
                );
                return Err(AppError::RuntimeSessionAlreadyActive);
            }

            super::runtime_lifecycle::stop_active_session(context).await?;
        }
        super::runtime_lifecycle::ActiveSessionState::Stale(session) => {
            tracing::warn!(
                session_id = session.id,
                "stale runtime session was reconciled before connect"
            );
        }
        super::runtime_lifecycle::ActiveSessionState::None => {}
    }

    let launch = resolve_launch(context, &config)?;
    let session_id = context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config.id),
            status: RuntimeSessionStatus::Starting,
            socks_host: launch.socks.as_ref().map(|inbound| inbound.host.clone()),
            socks_port: launch.socks.as_ref().map(|inbound| i64::from(inbound.port)),
            http_host: launch.http.as_ref().map(|inbound| inbound.host.clone()),
            http_port: launch.http.as_ref().map(|inbound| i64::from(inbound.port)),
            shadowsocks_host: launch
                .shadowsocks
                .as_ref()
                .map(|inbound| inbound.host.clone()),
            shadowsocks_port: launch
                .shadowsocks
                .as_ref()
                .map(|inbound| i64::from(inbound.port)),
            process_id: None,
            started_at: None,
            stopped_at: None,
        })
        .await?;

    let process = match xray_runtime::spawn_detached(
        &launch.binary_path,
        &context.runtime_paths.runtime_dir,
        session_id,
        &launch.config,
        &launch.ready_host,
        launch.ready_port,
        Duration::from_millis(defaults::DEFAULT_XRAY_STARTUP_TIMEOUT_MS),
    )
    .await
    {
        Ok(process) => process,
        Err(error) => {
            context
                .db
                .update_runtime_session_state(
                    session_id,
                    RuntimeSessionStatus::Failed,
                    None,
                    None,
                    Some(&super::runtime_lifecycle::now_string()),
                )
                .await?;
            return Err(error);
        }
    };

    context
        .db
        .update_runtime_session_state(
            session_id,
            RuntimeSessionStatus::Running,
            Some(i64::from(process.pid)),
            Some(&super::runtime_lifecycle::now_string()),
            None,
        )
        .await?;
    context.db.set_active_config(config.id).await?;

    println!("Connected config {}", config.id);
    if let Some(name) = &config.name {
        println!("Name: {name}");
    }
    if let Some(inbound) = &launch.socks {
        println!(
            "SOCKS: {}",
            super::runtime_output::format_inbound_endpoint(&inbound.host, inbound.port)
        );
    }
    if let Some(inbound) = &launch.http {
        println!(
            "HTTP: {}",
            super::runtime_output::format_inbound_endpoint(&inbound.host, inbound.port)
        );
    }
    if let Some(inbound) = &launch.shadowsocks {
        println!(
            "Shadowsocks: {}",
            super::runtime_output::format_inbound_endpoint(&inbound.host, inbound.port)
        );
    }
    println!("PID: {}", process.pid);
    println!("Runtime config: {}", process.paths.config_path.display());

    Ok(())
}

struct ResolvedLaunch {
    binary_path: std::path::PathBuf,
    config: crate::xray::XrayConfig,
    ready_host: String,
    ready_port: u16,
    socks: Option<ResolvedInbound>,
    http: Option<ResolvedInbound>,
    shadowsocks: Option<ResolvedInbound>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResolvedInbound {
    host: String,
    port: u16,
}

fn resolve_launch(
    context: &AppContext,
    config: &ConfigRecord,
) -> crate::app::Result<ResolvedLaunch> {
    let runtime = &context.app_config.runtime;
    let socks = runtime.socks.enabled.then_some((
        runtime.socks.host.as_str(),
        runtime.socks.port,
        runtime.socks.udp,
    ));
    let http = runtime
        .http
        .enabled
        .then_some((runtime.http.host.as_str(), runtime.http.port));
    let shadowsocks = if runtime.shadowsocks.enabled {
        Some((
            runtime.shadowsocks.host.as_str(),
            runtime.shadowsocks.port,
            runtime.shadowsocks.method.as_str(),
            runtime.shadowsocks.password.resolve()?,
            runtime.shadowsocks.network.as_str(),
        ))
    } else {
        None
    };

    if socks.is_none() && http.is_none() && shadowsocks.is_none() {
        return Err(AppError::NoRuntimeInboundEnabled);
    }

    let node = node_from_record(config)?;
    let mut xray_config = generate_runtime_config_for_inbounds(&node, socks, http)
        .map_err(AppError::InvalidArgument)?;
    if let Some((host, port, method, password, network)) = &shadowsocks {
        xray_config.inbounds.push(Inbound {
            tag: "shadowsocks-in".to_string(),
            port: *port,
            listen: (*host).to_string(),
            protocol: "shadowsocks".to_string(),
            settings: Some(serde_json::json!({
                "method": method,
                "password": password,
                "network": network
            })),
        });
    }

    let (ready_host, ready_port) = if let Some((host, port, _)) = socks {
        (connect_host_for_bind_host(host), port)
    } else if let Some((host, port)) = http {
        (connect_host_for_bind_host(host), port)
    } else if let Some((host, port, _, _, _)) = &shadowsocks {
        (connect_host_for_bind_host(host), *port)
    } else {
        unreachable!("validated at least one inbound")
    };
    let binary_path = match runtime.engine.as_str() {
        "xray" => context.runtime_paths.xray_path.clone(),
        "v2ray" => context.runtime_paths.v2ray_path.clone(),
        other => std::path::PathBuf::from(other),
    };

    Ok(ResolvedLaunch {
        binary_path,
        config: xray_config,
        ready_host,
        ready_port,
        socks: socks.map(|(host, port, _)| ResolvedInbound {
            host: host.to_string(),
            port,
        }),
        http: http.map(|(host, port)| ResolvedInbound {
            host: host.to_string(),
            port,
        }),
        shadowsocks: shadowsocks.map(|(host, port, _, _, _)| ResolvedInbound {
            host: host.to_string(),
            port,
        }),
    })
}

fn node_from_record(config: &ConfigRecord) -> crate::app::Result<Node> {
    let protocol = match config.protocol.as_str() {
        "vless" => Protocol::Vless,
        "vmess" => Protocol::Vmess,
        "ss" => Protocol::Ss,
        "trojan" => Protocol::Trojan,
        "http" => Protocol::Http,
        "socks5" => Protocol::Socks5,
        other => return Err(AppError::UnsupportedProtocol(other.to_string())),
    };

    Ok(Node {
        protocol,
        address: config.address.clone(),
        port: config.port as u16,
        username: config.username.clone(),
        uuid: config.uuid.clone(),
        password: config.password.clone(),
        method: config.method.clone(),
        network: config.network.clone(),
        tls: config.tls.clone(),
        sni: config.sni.clone(),
        host: config.host.clone(),
        path: config.path.clone(),
        name: config.name.clone(),
        raw_config: config.raw_config.clone(),
    })
}

fn connect_host_for_bind_host(host: &str) -> String {
    match host {
        "0.0.0.0" => "127.0.0.1".to_string(),
        "::" => "::1".to_string(),
        _ => host.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_protocol() {
        let record = ConfigRecord {
            id: 1,
            subscription_id: None,
            dedup_key: "key".to_string(),
            protocol: "unknown".to_string(),
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: None,
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: None,
            sni: None,
            host: None,
            path: None,
            name: None,
            raw_config: "raw".to_string(),
            is_active: false,
            is_enabled: true,
            is_selected: false,
            imported_at: "now".to_string(),
            created_at: "now".to_string(),
            updated_at: "now".to_string(),
        };

        assert!(matches!(
            node_from_record(&record),
            Err(AppError::UnsupportedProtocol(_))
        ));
    }

    #[test]
    fn maps_wildcard_bind_hosts_to_loopback_for_readiness() {
        assert_eq!(connect_host_for_bind_host("0.0.0.0"), "127.0.0.1");
        assert_eq!(connect_host_for_bind_host("::"), "::1");
        assert_eq!(connect_host_for_bind_host("127.0.0.1"), "127.0.0.1");
    }
}
