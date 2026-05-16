use serde_json::json;

use crate::model::Node;

use super::{Inbound, LogConfig, XrayConfig, outbound::node_to_outbound};

#[cfg(test)]
mod tests;

pub fn generate_probe_config(node: &Node, local_port: u16) -> Result<XrayConfig, String> {
    let inbound = Inbound {
        tag: "probe-in".to_string(),
        port: local_port,
        listen: "127.0.0.1".to_string(),
        protocol: "socks".to_string(),
        settings: Some(json!({"udp": false})),
    };

    let outbound = node_to_outbound(node, "proxy")?;

    Ok(XrayConfig {
        log: LogConfig {
            loglevel: "warning".to_string(),
        },
        inbounds: vec![inbound],
        outbounds: vec![outbound],
    })
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
    let inbounds = build_inbounds(socks, http);
    let outbound = node_to_outbound(node, "proxy")?;

    Ok(XrayConfig {
        log: LogConfig {
            loglevel: "warning".to_string(),
        },
        inbounds,
        outbounds: vec![outbound],
    })
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
