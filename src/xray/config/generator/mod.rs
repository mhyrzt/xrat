use serde_json::json;

use crate::model::Node;

use super::routing::{apply_runtime_routing, field_rule};
use super::tuning::{XrayGenOptions, apply_runtime_tuning};
use super::{Inbound, LogConfig, XrayConfig, outbound::node_to_outbound};

#[cfg(test)]
mod tests;

pub fn generate_probe_config(node: &Node, local_port: u16) -> Result<XrayConfig, String> {
    generate_probe_config_with_options(node, local_port, &XrayGenOptions::default())
}

pub fn generate_probe_config_with_options(
    node: &Node,
    local_port: u16,
    options: &XrayGenOptions,
) -> Result<XrayConfig, String> {
    let inbound = Inbound {
        tag: "probe-in".to_string(),
        port: local_port,
        listen: "127.0.0.1".to_string(),
        protocol: "socks".to_string(),
        settings: Some(json!({"udp": false})),
    };

    let outbound = node_to_outbound(node, "proxy")?;

    let mut config = XrayConfig {
        log: LogConfig {
            loglevel: "warning".to_string(),
        },
        inbounds: vec![inbound],
        outbounds: vec![outbound],
        api: None,
        stats: None,
        policy: None,
        routing: None,
    };
    apply_runtime_tuning(&mut config, options);
    Ok(config)
}

pub fn generate_runtime_config(
    node: &Node,
    socks_port: u16,
    http_port: Option<u16>,
) -> Result<XrayConfig, String> {
    generate_runtime_config_with_inbounds(node, "127.0.0.1", socks_port, None, http_port)
}

pub fn generate_runtime_config_with_inbounds(
    node: &Node,
    socks_host: &str,
    socks_port: u16,
    http_host: Option<&str>,
    http_port: Option<u16>,
) -> Result<XrayConfig, String> {
    generate_runtime_config_for_inbounds(
        node,
        Some((socks_host, socks_port, true)),
        http_port.map(|port| (http_host.unwrap_or(socks_host), port)),
    )
}

pub fn generate_runtime_config_for_inbounds(
    node: &Node,
    socks: Option<(&str, u16, bool)>,
    http: Option<(&str, u16)>,
) -> Result<XrayConfig, String> {
    generate_runtime_config_for_inbounds_with_options(node, socks, http, &XrayGenOptions::default())
}

pub fn generate_runtime_config_for_inbounds_with_options(
    node: &Node,
    socks: Option<(&str, u16, bool)>,
    http: Option<(&str, u16)>,
    options: &XrayGenOptions,
) -> Result<XrayConfig, String> {
    let inbounds = build_inbounds(socks, http);
    let outbound = node_to_outbound(node, "proxy")?;

    let mut config = XrayConfig {
        log: LogConfig {
            loglevel: "warning".to_string(),
        },
        inbounds,
        outbounds: vec![outbound],
        api: None,
        stats: None,
        policy: None,
        routing: None,
    };
    apply_runtime_routing(&mut config, options.routing.as_ref());
    apply_runtime_tuning(&mut config, options);
    Ok(config)
}

/// Enable the xray gRPC StatsService on an already-built runtime config. Adds a
/// `dokodemo-door` inbound tagged `api`, the `api`/`stats`/`policy` objects, and
/// a routing rule that dispatches the api inbound to the api handler. Counters
/// are enabled for system inbound/outbound traffic so totals are available.
pub fn enable_stats_api(config: &mut XrayConfig, host: &str, port: u16) {
    use crate::xray::parsing::core::{ApiObject, ApiServiceName, PolicyObject, SystemPolicyObject};

    config.inbounds.push(Inbound {
        tag: "api".to_string(),
        port,
        listen: host.to_string(),
        protocol: "dokodemo-door".to_string(),
        settings: Some(json!({ "address": host })),
    });
    config.api = Some(ApiObject {
        tag: "api".to_string(),
        listen: None,
        services: vec![ApiServiceName::StatsService],
    });
    config.stats = Some(json!({}));
    config.policy = Some(PolicyObject {
        levels: None,
        system: Some(SystemPolicyObject {
            stats_inbound_uplink: Some(true),
            stats_inbound_downlink: Some(true),
            stats_outbound_uplink: Some(true),
            stats_outbound_downlink: Some(true),
        }),
    });
    let routing = config.routing.get_or_insert_with(|| super::RoutingConfig {
        domain_strategy: None,
        rules: Vec::new(),
    });
    routing.rules.insert(
        0,
        field_rule(None, None, Some(vec!["api".to_string()]), "api"),
    );
}

fn build_inbounds(socks: Option<(&str, u16, bool)>, http: Option<(&str, u16)>) -> Vec<Inbound> {
    let mut inbounds = Vec::new();

    if let Some((host, port, udp)) = socks {
        inbounds.push(Inbound {
            tag: "socks-in".to_string(),
            port,
            listen: host.to_string(),
            protocol: "socks".to_string(),
            settings: Some(json!({"udp": udp})),
        });
    }

    if let Some((host, port)) = http {
        inbounds.push(Inbound {
            tag: "http-in".to_string(),
            port,
            listen: host.to_string(),
            protocol: "http".to_string(),
            settings: None,
        });
    }

    inbounds
}
