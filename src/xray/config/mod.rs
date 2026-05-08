mod outbound;
mod stream;
mod types;

use serde_json::json;

use outbound::node_to_outbound;

use crate::model::Node;

pub use types::{
    GrpcSettings, Inbound, LogConfig, Outbound, StreamSettings, TcpSettings, TlsSettings,
    WsSettings, XrayConfig,
};

pub fn generate_probe_config(node: &Node, local_port: u16) -> Result<XrayConfig, String> {
    let inbound = Inbound {
        tag: "probe-in".to_string(),
        port: local_port,
        listen: "127.0.0.1".to_string(),
        protocol: "socks".to_string(),
        settings: Some(json!({
            "udp": false
        })),
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
    let mut inbounds = Vec::new();

    if let Some((host, port, udp)) = socks {
        inbounds.push(Inbound {
            tag: "socks-in".to_string(),
            port,
            listen: host.to_string(),
            protocol: "socks".to_string(),
            settings: Some(json!({
                "udp": udp
            })),
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

    let outbound = node_to_outbound(node, "proxy")?;

    Ok(XrayConfig {
        log: LogConfig {
            loglevel: "warning".to_string(),
        },
        inbounds,
        outbounds: vec![outbound],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Protocol;

    #[test]
    fn test_generate_vless_probe_config() {
        let node = Node {
            protocol: Protocol::Vless,
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: Some("test-uuid".to_string()),
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: Some("tls".to_string()),
            sni: Some("example.com".to_string()),
            host: None,
            path: None,
            name: Some("test".to_string()),
            extensions: None,
            raw_config: "".to_string(),
        };

        let config = generate_probe_config(&node, 10808).unwrap();
        assert_eq!(config.inbounds.len(), 1);
        assert_eq!(config.inbounds[0].port, 10808);
        assert_eq!(config.outbounds.len(), 1);
        assert_eq!(config.outbounds[0].protocol, "vless");
    }

    #[test]
    fn test_generate_vmess_ws_config() {
        let node = Node {
            protocol: Protocol::Vmess,
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: Some("test-uuid".to_string()),
            password: None,
            method: None,
            network: "ws".to_string(),
            tls: Some("tls".to_string()),
            sni: None,
            host: Some("example.com".to_string()),
            path: Some("/path".to_string()),
            name: Some("test".to_string()),
            extensions: None,
            raw_config: "".to_string(),
        };

        let config = generate_runtime_config(&node, 1080, Some(8080)).unwrap();
        assert_eq!(config.inbounds.len(), 2);
        assert_eq!(config.outbounds[0].protocol, "vmess");

        let stream = config.outbounds[0].stream_settings.as_ref().unwrap();
        assert_eq!(stream.network, "ws");
        assert!(stream.ws_settings.is_some());
    }

    #[test]
    fn generates_http_only_runtime_config() {
        let node = Node {
            protocol: Protocol::Http,
            address: "example.com".to_string(),
            port: 8080,
            username: Some("user".to_string()),
            uuid: None,
            password: Some("pass".to_string()),
            method: None,
            network: "tcp".to_string(),
            tls: None,
            sni: None,
            host: None,
            path: None,
            name: Some("http".to_string()),
            extensions: None,
            raw_config: "".to_string(),
        };

        let config =
            generate_runtime_config_for_inbounds(&node, None, Some(("127.0.0.1", 18080))).unwrap();

        assert_eq!(config.inbounds.len(), 1);
        assert_eq!(config.inbounds[0].protocol, "http");
        assert_eq!(config.inbounds[0].port, 18080);
    }
}
