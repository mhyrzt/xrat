use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use crate::model::{Node, Protocol};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrayConfig {
    pub log: LogConfig,
    pub inbounds: Vec<Inbound>,
    pub outbounds: Vec<Outbound>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub loglevel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inbound {
    pub tag: String,
    pub port: u16,
    pub listen: String,
    pub protocol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outbound {
    pub tag: String,
    pub protocol: String,
    pub settings: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_settings: Option<StreamSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamSettings {
    pub network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_settings: Option<TlsSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ws_settings: Option<WsSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_settings: Option<TcpSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grpc_settings: Option<GrpcSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TlsSettings {
    pub server_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_insecure: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsSettings {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrpcSettings {
    pub service_name: String,
}

/// Generate a temporary Xray config for connection testing (probe mode)
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

/// Generate a full runtime Xray config for long-lived sessions
pub fn generate_runtime_config(
    node: &Node,
    socks_port: u16,
    http_port: Option<u16>,
) -> Result<XrayConfig, String> {
    let mut inbounds = vec![Inbound {
        tag: "socks-in".to_string(),
        port: socks_port,
        listen: "127.0.0.1".to_string(),
        protocol: "socks".to_string(),
        settings: Some(json!({
            "udp": true
        })),
    }];

    if let Some(port) = http_port {
        inbounds.push(Inbound {
            tag: "http-in".to_string(),
            port,
            listen: "127.0.0.1".to_string(),
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

fn node_to_outbound(node: &Node, tag: &str) -> Result<Outbound, String> {
    let protocol = node.protocol.as_str().to_string();
    let settings = build_outbound_settings(node)?;
    let stream_settings = build_stream_settings(node)?;

    Ok(Outbound {
        tag: tag.to_string(),
        protocol,
        settings,
        stream_settings,
    })
}

fn build_outbound_settings(node: &Node) -> Result<serde_json::Value, String> {
    match node.protocol {
        Protocol::Vless => {
            let uuid = node.uuid.as_ref().ok_or("vless requires uuid")?;
            Ok(json!({
                "vnext": [{
                    "address": node.address,
                    "port": node.port,
                    "users": [{
                        "id": uuid,
                        "encryption": "none"
                    }]
                }]
            }))
        }
        Protocol::Vmess => {
            let uuid = node.uuid.as_ref().ok_or("vmess requires uuid")?;
            Ok(json!({
                "vnext": [{
                    "address": node.address,
                    "port": node.port,
                    "users": [{
                        "id": uuid,
                        "alterId": 0,
                        "security": "auto"
                    }]
                }]
            }))
        }
        Protocol::Trojan => {
            let password = node.password.as_ref().ok_or("trojan requires password")?;
            Ok(json!({
                "servers": [{
                    "address": node.address,
                    "port": node.port,
                    "password": password
                }]
            }))
        }
        Protocol::Ss => {
            let password = node
                .password
                .as_ref()
                .ok_or("shadowsocks requires password")?;
            let method = node.method.as_ref().ok_or("shadowsocks requires method")?;
            Ok(json!({
                "servers": [{
                    "address": node.address,
                    "port": node.port,
                    "method": method,
                    "password": password
                }]
            }))
        }
        Protocol::Socks5 => {
            let mut server = json!({
                "address": node.address,
                "port": node.port
            });
            if let Some(username) = &node.username {
                if let Some(password) = &node.password {
                    server["users"] = json!([{
                        "user": username,
                        "pass": password
                    }]);
                }
            }
            Ok(json!({
                "servers": [server]
            }))
        }
        Protocol::Http => {
            let mut server = json!({
                "address": node.address,
                "port": node.port
            });
            if let Some(username) = &node.username {
                if let Some(password) = &node.password {
                    server["users"] = json!([{
                        "user": username,
                        "pass": password
                    }]);
                }
            }
            Ok(json!({
                "servers": [server]
            }))
        }
    }
}

fn build_stream_settings(node: &Node) -> Result<Option<StreamSettings>, String> {
    let network = node.network.as_str();

    // Simple protocols don't need stream settings
    if matches!(node.protocol, Protocol::Socks5 | Protocol::Http) {
        return Ok(None);
    }

    let security = node.tls.as_ref().map(|s| s.to_string());
    let tls_settings = if node.tls.as_deref() == Some("tls") {
        Some(TlsSettings {
            server_name: node.sni.clone().unwrap_or_else(|| node.address.clone()),
            allow_insecure: None,
        })
    } else {
        None
    };

    let ws_settings = if network == "ws" {
        let mut headers = HashMap::new();
        if let Some(host) = &node.host {
            headers.insert("Host".to_string(), host.clone());
        }
        Some(WsSettings {
            path: node.path.clone().unwrap_or_else(|| "/".to_string()),
            headers: if headers.is_empty() {
                None
            } else {
                Some(headers)
            },
        })
    } else {
        None
    };

    let grpc_settings = if network == "grpc" {
        Some(GrpcSettings {
            service_name: node.path.clone().unwrap_or_default(),
        })
    } else {
        None
    };

    let tcp_settings = if network == "tcp" && node.path.is_some() {
        Some(TcpSettings {
            header: Some(json!({
                "type": "http",
                "request": {
                    "path": [node.path.as_ref().unwrap()]
                }
            })),
        })
    } else {
        None
    };

    Ok(Some(StreamSettings {
        network: network.to_string(),
        security,
        tls_settings,
        ws_settings,
        tcp_settings,
        grpc_settings,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

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
            raw_config: "".to_string(),
        };

        let config = generate_runtime_config(&node, 1080, Some(8080)).unwrap();
        assert_eq!(config.inbounds.len(), 2);
        assert_eq!(config.outbounds[0].protocol, "vmess");

        let stream = config.outbounds[0].stream_settings.as_ref().unwrap();
        assert_eq!(stream.network, "ws");
        assert!(stream.ws_settings.is_some());
    }
}
