use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::model::Node;

pub(super) fn build_tls_and_transport(
    node: &Node,
    extensions: &mut BTreeMap<String, Value>,
) -> Result<(Option<Value>, Option<Value>), String> {
    let tls = match node.tls.as_deref().unwrap_or("none") {
        "" | "none" => {
            if node.sni.is_some()
                || ["insecure", "allowInsecure", "alpn", "fp", "pbk", "sid"]
                    .iter()
                    .any(|key| extensions.contains_key(*key))
            {
                return Err(
                    "sing-box TLS fields require security=tls or security=reality".to_string(),
                );
            }
            None
        }
        "tls" | "reality" => {
            let mut options = json!({
                "enabled": true,
                "server_name": node.sni.as_deref().unwrap_or(&node.address),
            });
            if let Some(insecure) = take_bool_alias(extensions, "allowInsecure", "insecure")? {
                options["insecure"] = json!(insecure);
            }
            if let Some(alpn) = take_string(extensions, "alpn")? {
                let values: Vec<&str> = alpn.split(',').map(str::trim).collect();
                if values.iter().any(|value| value.is_empty()) {
                    return Err("link parameter \"alpn\" contains an empty protocol".to_string());
                }
                options["alpn"] = json!(values);
            }
            if let Some(fingerprint) = take_string(extensions, "fp")? {
                if !matches!(
                    fingerprint.as_str(),
                    "chrome"
                        | "firefox"
                        | "edge"
                        | "safari"
                        | "360"
                        | "qq"
                        | "ios"
                        | "android"
                        | "random"
                        | "randomized"
                ) {
                    return Err(format!(
                        "unsupported sing-box uTLS fingerprint {fingerprint:?}"
                    ));
                }
                options["utls"] = json!({"enabled": true, "fingerprint": fingerprint});
            }
            if node.tls.as_deref() == Some("reality") {
                let public_key = take_string(extensions, "pbk")?
                    .or(take_string(extensions, "password")?)
                    .filter(|value| !value.is_empty())
                    .ok_or("REALITY requires pbk/password public key")?;
                let short_id = take_string(extensions, "sid")?.unwrap_or_default();
                if short_id.len() > 16
                    || short_id.len() % 2 != 0
                    || !short_id.chars().all(|item| item.is_ascii_hexdigit())
                {
                    return Err(format!(
                        "REALITY short id must be 0-16 hex digits; got {short_id:?}"
                    ));
                }
                // sing-box refuses a REALITY client without uTLS.
                if options.get("utls").is_none() {
                    options["utls"] = json!({"enabled": true, "fingerprint": "chrome"});
                }
                options["reality"] =
                    json!({"enabled": true, "public_key": public_key, "short_id": short_id});
            }
            Some(options)
        }
        other => return Err(format!("unsupported sing-box TLS security {other:?}")),
    };

    let transport = match node.network.as_str() {
        "" | "tcp" | "raw" => {
            if node.host.is_some() || node.path.is_some() {
                return Err("sing-box plain TCP cannot preserve host or path".to_string());
            }
            None
        }
        "ws" | "websocket" => {
            let mut value = json!({"type": "ws", "path": node.path.as_deref().unwrap_or("/")});
            if let Some(host) = &node.host {
                value["headers"] = json!({"Host": host});
            }
            Some(value)
        }
        "grpc" => {
            let service = take_string(extensions, "serviceName")?
                .or_else(|| node.path.clone())
                .unwrap_or_default();
            Some(json!({"type": "grpc", "service_name": service}))
        }
        "httpupgrade" => {
            let mut value =
                json!({"type": "httpupgrade", "path": node.path.as_deref().unwrap_or("/")});
            if let Some(host) = &node.host {
                value["host"] = json!(host);
            }
            Some(value)
        }
        "http" | "h2" => {
            let mut value = json!({"type": "http", "path": node.path.as_deref().unwrap_or("/")});
            if let Some(host) = &node.host {
                value["host"] = json!([host]);
            }
            Some(value)
        }
        "quic" => Some(json!({"type": "quic"})),
        other => return Err(format!("unsupported sing-box transport {other:?}")),
    };
    Ok((tls, transport))
}

pub(super) fn take_string(
    extensions: &mut BTreeMap<String, Value>,
    key: &str,
) -> Result<Option<String>, String> {
    let Some(value) = extensions.remove(key) else {
        return Ok(None);
    };
    match value {
        Value::String(value) => Ok(Some(value)),
        Value::Number(value) => Ok(Some(value.to_string())),
        Value::Bool(value) => Ok(Some(value.to_string())),
        _ => Err(format!("link parameter {key:?} must be a scalar value")),
    }
}

fn take_bool_alias(
    extensions: &mut BTreeMap<String, Value>,
    first: &str,
    second: &str,
) -> Result<Option<bool>, String> {
    let first_value = take_string(extensions, first)?;
    let second_value = take_string(extensions, second)?;
    if first_value.is_some() && second_value.is_some() && first_value != second_value {
        return Err(format!(
            "conflicting link parameters {first:?} and {second:?}"
        ));
    }
    match first_value.or(second_value).as_deref() {
        None => Ok(None),
        Some("1" | "true") => Ok(Some(true)),
        Some("0" | "false") => Ok(Some(false)),
        _ => Err(format!(
            "link parameter {first:?}/{second:?} must be a boolean"
        )),
    }
}
