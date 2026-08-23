use serde_json::json;
use std::collections::HashMap;

use super::extensions::ExtensionResolver;
use super::types::{
    GrpcSettings, HttpUpgradeSettings, KcpSettings, RawSettings, RealitySettings, StreamSettings,
    TlsSettings, WsSettings, XhttpSettings,
};
use crate::model::{Node, Protocol};

pub(super) fn build_stream_settings(
    node: &Node,
    extensions: &mut ExtensionResolver,
) -> Result<Option<StreamSettings>, String> {
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
        let alpn = extensions.string("alpn")?.map(|value| {
            value
                .split(',')
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
        });
        Some(TlsSettings {
            server_name: node.sni.clone().unwrap_or_else(|| node.address.clone()),
            allow_insecure: extensions.alias_bool("allowInsecure", &["insecure"])?,
            fingerprint: extensions.string("fp")?,
            alpn: alpn.filter(|parts| !parts.is_empty()),
        })
    } else {
        None
    };

    let reality_settings = if node.tls.as_deref() == Some("reality") {
        let public_key = extensions
            .alias_string("password", &["pbk"])?
            .filter(|value| !value.is_empty())
            .ok_or("REALITY requires password (link parameter pbk/password)")?;
        Some(RealitySettings {
            server_name: node.sni.clone().unwrap_or_else(|| node.address.clone()),
            public_key,
            short_id: extensions.string("sid")?,
            spider_x: extensions.string("spx")?,
            fingerprint: Some(
                extensions
                    .string("fp")?
                    .unwrap_or_else(|| "chrome".to_string()),
            ),
            mldsa65_verify: extensions.string("mldsa65Verify")?,
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
            heartbeat_period: extensions.u64("heartbeatPeriod")?,
            max_early_data: extensions.u64("ed")?,
            early_data_header_name: extensions.string("eh")?,
        })
    } else {
        None
    };

    let grpc_settings = if network == "grpc" {
        let mode = extensions.string("mode")?;
        Some(GrpcSettings {
            service_name: extensions
                .string("serviceName")?
                .or_else(|| node.path.clone())
                .unwrap_or_default(),
            authority: extensions.string("authority")?,
            multi_mode: extensions
                .boolean("multiMode")?
                .or_else(|| mode.map(|value| value == "multi")),
            idle_timeout: extensions.alias_u64("idleTimeout", &["idle_timeout"])?,
        })
    } else {
        None
    };

    let raw_header_type = if network == "raw" {
        extensions.alias_string("headerType", &["type"])?
    } else {
        None
    };
    if let Some(header_type) = raw_header_type.as_deref()
        && !matches!(header_type, "none" | "http")
    {
        return Err(format!("unsupported Xray raw header type {header_type:?}"));
    }
    let raw_settings = if network == "raw" && raw_header_type.as_deref() == Some("http") {
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
        let mode = extensions.string("mode")?;
        Some(XhttpSettings {
            host: node.host.clone(),
            path: node.path.clone().unwrap_or_else(|| "/".to_string()),
            extra: build_xhttp_extra(node, mode.as_deref(), extensions)?,
            mode,
        })
    } else {
        None
    };

    let kcp_settings = if network == "mkcp" {
        if extensions.value("seed").is_some() || extensions.value("headerType").is_some() {
            return Err("current Xray mKCP no longer supports seed/headerType".to_string());
        }
        Some(KcpSettings {
            mtu: extensions.u64("mtu")?,
            tti: extensions.u64("tti")?,
            uplink_capacity: extensions.u64("uplinkCapacity")?,
            downlink_capacity: extensions.u64("downlinkCapacity")?,
            congestion: extensions.boolean("congestion")?,
            read_buffer_size: extensions.u64("readBufferSize")?,
            write_buffer_size: extensions.u64("writeBufferSize")?,
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

fn build_xhttp_extra(
    node: &Node,
    mode: Option<&str>,
    extensions: &mut ExtensionResolver,
) -> Result<Option<serde_json::Value>, String> {
    let mut extra = serde_json::Map::new();

    if let Some(value) = extensions.alias_string("x_padding_bytes", &["x_padding bytes"])? {
        extra.insert("xPaddingBytes".to_string(), json!(value));
    }
    if let Some(official) = extensions.object("extra")? {
        extra.extend(official);
    }

    for key in [
        "xPaddingBytes",
        "xPaddingKey",
        "xPaddingHeader",
        "xPaddingPlacement",
        "xPaddingMethod",
        "uplinkHTTPMethod",
        "sessionIDPlacement",
        "sessionIDKey",
        "sessionIDTable",
        "sessionIDLength",
        "seqPlacement",
        "seqKey",
        "uplinkDataPlacement",
        "uplinkDataKey",
        "uplinkChunkSize",
        "scMaxEachPostBytes",
        "scMinPostsIntervalMs",
        "scStreamUpServerSecs",
    ] {
        if let Some(value) = extensions.string(key)? {
            extra.insert(key.to_string(), json!(value));
        }
    }
    for key in ["xPaddingObfsMode", "noGRPCHeader", "noSSEHeader"] {
        if let Some(value) = extensions.boolean(key)? {
            extra.insert(key.to_string(), json!(value));
        }
    }
    for key in ["scMaxBufferedPosts", "serverMaxHeaderBytes"] {
        if let Some(value) = extensions.i64(key)? {
            extra.insert(key.to_string(), json!(value));
        }
    }
    for key in ["headers", "xmux", "downloadSettings"] {
        if let Some(value) = extensions.object(key)? {
            extra.insert(key.to_string(), serde_json::Value::Object(value));
        }
    }

    remove_matching_structural(&mut extra, "host", node.host.as_deref())?;
    remove_matching_structural(
        &mut extra,
        "path",
        Some(node.path.as_deref().unwrap_or("/")),
    )?;
    remove_matching_structural(&mut extra, "mode", mode)?;

    Ok((!extra.is_empty()).then_some(serde_json::Value::Object(extra)))
}

fn remove_matching_structural(
    extra: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    outer: Option<&str>,
) -> Result<(), String> {
    let Some(value) = extra.remove(key) else {
        return Ok(());
    };
    if value.as_str() == outer {
        return Ok(());
    }
    Err(format!(
        "XHTTP `extra.{key}` conflicts with the outer link parameter"
    ))
}
