use serde_json::json;
use std::collections::HashMap;

use super::types::{GrpcSettings, StreamSettings, TcpSettings, TlsSettings, WsSettings};
use crate::model::{Node, Protocol};

pub(super) fn build_stream_settings(node: &Node) -> Result<Option<StreamSettings>, String> {
    let network = node.network.as_str();

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

    let tcp_settings = if network == "tcp" {
        node.path.as_ref().map(|path| TcpSettings {
            header: Some(json!({
                "type": "http",
                "request": {
                    "path": [path]
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
