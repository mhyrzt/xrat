use serde_json::json;
use std::collections::HashMap;

use super::types::{
    GrpcSettings, HttpUpgradeSettings, KcpSettings, RawSettings, RealitySettings, StreamSettings,
    TlsSettings, WsSettings, XhttpSettings,
};
use crate::model::{Node, Protocol};

fn extension(node: &Node, key: &str) -> Option<String> {
    node.extension_string(key)
}

fn extension_bool(node: &Node, key: &str) -> Option<bool> {
    match node.extension_value(key)? {
        serde_json::Value::Bool(value) => Some(*value),
        serde_json::Value::Number(value) => value.as_i64().map(|value| value != 0),
        serde_json::Value::String(value) => match value.to_ascii_lowercase().as_str() {
            "1" | "true" => Some(true),
            "0" | "false" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

fn extension_u64(node: &Node, key: &str) -> Option<u64> {
    node.extension_value(key).and_then(|value| {
        value
            .as_u64()
            .or_else(|| value.as_str().and_then(|value| value.parse().ok()))
    })
}

fn validate_extensions(node: &Node) -> Result<(), String> {
    let Some(extensions) = &node.extensions else {
        return Ok(());
    };
    const SUPPORTED: &[&str] = &[
        "aid",
        "allowInsecure",
        "alpn",
        "authority",
        "congestion",
        "downlinkCapacity",
        "ed",
        "eh",
        "encryption",
        "extra",
        "flow",
        "fp",
        "headerType",
        "heartbeatPeriod",
        "idleTimeout",
        "idle_timeout",
        "insecure",
        "mldsa65Verify",
        "mode",
        "mtu",
        "multiMode",
        "password",
        "pbk",
        "readBufferSize",
        "scy",
        "security",
        "serviceName",
        "sid",
        "spx",
        "tti",
        "type",
        "uplinkCapacity",
        "v",
        "writeBufferSize",
    ];
    const METADATA: &[&str] = &["email", "group", "name", "remark", "remarks"];
    for key in extensions.keys() {
        if !SUPPORTED.contains(&key.as_str()) && !METADATA.contains(&key.as_str()) {
            return Err(format!(
                "unsupported link parameter {key:?}; refusing to generate a potentially incomplete runtime config"
            ));
        }
    }
    Ok(())
}

pub(super) fn build_stream_settings(node: &Node) -> Result<Option<StreamSettings>, String> {
    let network = match node.network.to_ascii_lowercase().as_str() {
        "" | "tcp" | "raw" => "raw",
        "ws" | "websocket" => "websocket",
        "kcp" | "mkcp" => "mkcp",
        "splithttp" | "xhttp" => "xhttp",
        "grpc" => "grpc",
        "httpupgrade" => "httpupgrade",
        "hysteria" => "hysteria",
        "http" | "h2" | "h3" | "quic" => {
            return Err(format!("removed Xray transport method {:?}", node.network));
        }
        _ => {
            return Err(format!(
                "unsupported Xray transport method {:?}",
                node.network
            ));
        }
    };

    if matches!(node.protocol, Protocol::Socks5 | Protocol::Http) {
        return Ok(None);
    }
    validate_extensions(node)?;

    let security = match node.tls.as_deref().unwrap_or("none") {
        "" | "none" => Some("none".to_string()),
        "tls" => Some("tls".to_string()),
        "reality" => {
            if !matches!(network, "raw" | "xhttp" | "grpc") {
                return Err(format!(
                    "REALITY is incompatible with Xray transport method {network:?}"
                ));
            }
            Some("reality".to_string())
        }
        value => return Err(format!("unsupported Xray transport security {value:?}")),
    };
    let tls_settings = if node.tls.as_deref() == Some("tls") {
        let alpn = extension(node, "alpn").map(|value| {
            value
                .split(',')
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
        });
        Some(TlsSettings {
            server_name: node.sni.clone().unwrap_or_else(|| node.address.clone()),
            allow_insecure: extension_bool(node, "allowInsecure")
                .or_else(|| extension_bool(node, "insecure")),
            fingerprint: extension(node, "fp"),
            alpn: alpn.filter(|parts| !parts.is_empty()),
        })
    } else {
        None
    };

    let reality_settings = if node.tls.as_deref() == Some("reality") {
        let public_key = extension(node, "password")
            .or_else(|| extension(node, "pbk"))
            .filter(|value| !value.is_empty())
            .ok_or("REALITY requires password (link parameter pbk/password)")?;
        Some(RealitySettings {
            server_name: node.sni.clone().unwrap_or_else(|| node.address.clone()),
            public_key,
            short_id: extension(node, "sid"),
            spider_x: extension(node, "spx"),
            fingerprint: Some(extension(node, "fp").unwrap_or_else(|| "chrome".to_string())),
            mldsa65_verify: extension(node, "mldsa65Verify"),
        })
    } else {
        None
    };

    let ws_settings = if network == "websocket" {
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
            heartbeat_period: extension_u64(node, "heartbeatPeriod"),
            max_early_data: extension_u64(node, "ed"),
            early_data_header_name: extension(node, "eh"),
        })
    } else {
        None
    };

    let grpc_settings = if network == "grpc" {
        Some(GrpcSettings {
            service_name: extension(node, "serviceName")
                .or_else(|| node.path.clone())
                .unwrap_or_default(),
            authority: extension(node, "authority"),
            multi_mode: extension_bool(node, "multiMode")
                .or_else(|| extension(node, "mode").map(|value| value == "multi")),
            idle_timeout: extension_u64(node, "idle_timeout")
                .or_else(|| extension_u64(node, "idleTimeout")),
        })
    } else {
        None
    };

    let raw_settings =
        if network == "raw" && extension(node, "headerType").as_deref() == Some("http") {
            Some(RawSettings {
                header: Some(json!({
                    "type": "http",
                    "request": {
                        "path": [node.path.as_deref().unwrap_or("/")]
                    }
                })),
            })
        } else {
            None
        };

    let xhttp_settings = if network == "xhttp" {
        Some(XhttpSettings {
            host: node.host.clone(),
            path: node.path.clone().unwrap_or_else(|| "/".to_string()),
            mode: extension(node, "mode"),
            extra: node.extension_value("extra").cloned(),
        })
    } else {
        None
    };

    let kcp_settings = if network == "mkcp" {
        if node.extension_value("seed").is_some() || node.extension_value("headerType").is_some() {
            return Err("current Xray mKCP no longer supports seed/headerType".to_string());
        }
        Some(KcpSettings {
            mtu: extension_u64(node, "mtu"),
            tti: extension_u64(node, "tti"),
            uplink_capacity: extension_u64(node, "uplinkCapacity"),
            downlink_capacity: extension_u64(node, "downlinkCapacity"),
            congestion: extension_bool(node, "congestion"),
            read_buffer_size: extension_u64(node, "readBufferSize"),
            write_buffer_size: extension_u64(node, "writeBufferSize"),
        })
    } else {
        None
    };

    let httpupgrade_settings = (network == "httpupgrade").then(|| HttpUpgradeSettings {
        path: node.path.clone().unwrap_or_else(|| "/".to_string()),
        host: node.host.clone(),
        headers: None,
    });

    if network == "hysteria" {
        return Err("Xray Hysteria links require complete hysteriaSettings and are not representable by common share links".to_string());
    }

    Ok(Some(StreamSettings {
        network: network.to_string(),
        security,
        tls_settings,
        reality_settings,
        ws_settings,
        raw_settings,
        kcp_settings,
        grpc_settings,
        xhttp_settings,
        httpupgrade_settings,
        sockopt: None,
    }))
}
