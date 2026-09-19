use std::collections::BTreeMap;

use url::Url;

use crate::model::Node;

const SUPPORTED_OPTIONS: &[&str] = &[
    "insecure",
    "alpn",
    "obfs",
    "obfs-password",
    "upmbps",
    "downmbps",
];

pub fn build_hy2_outbound(node: &Node) -> Result<serde_json::Value, String> {
    let password = node
        .password
        .as_deref()
        .filter(|password| !password.is_empty())
        .ok_or_else(|| "Hysteria2 authentication password is required for sing-box".to_string())?;
    let network = match node.network.as_str() {
        "tcp" | "udp" => node.network.as_str(),
        value => {
            return Err(format!(
                "Hysteria2 network must be tcp or udp for sing-box; got {value:?}"
            ));
        }
    };
    let mut outbound = serde_json::json!({
        "type": "hysteria2",
        "tag": "proxy",
        "server": node.address,
        "server_port": node.port,
        "password": password,
        "network": network,
        "tls": {
            "enabled": true,
            "server_name": node.sni.as_deref().unwrap_or(&node.address)
        }
    });

    let options = hy2_options(node)?;
    for (key, value) in &options {
        if !SUPPORTED_OPTIONS.contains(&key.as_str()) {
            return Err(format!(
                "Hysteria2 option {key:?} is not supported by sing-box 1.13; remove it or use Xray/V2Ray"
            ));
        }
        match key.as_str() {
            "insecure" => apply_insecure(&mut outbound, value)?,
            "alpn" => apply_alpn(&mut outbound, value)?,
            "obfs" => apply_obfs(&mut outbound, value)?,
            "obfs-password" => {
                apply_obfs_password(&mut outbound, value, options.contains_key("obfs"))?
            }
            "upmbps" => apply_mbps(&mut outbound, "up_mbps", value)?,
            "downmbps" => apply_mbps(&mut outbound, "down_mbps", value)?,
            _ => unreachable!("supported options were checked above"),
        }
    }

    Ok(outbound)
}

fn hy2_options(node: &Node) -> Result<BTreeMap<String, String>, String> {
    if let Some(extensions) = &node.extensions {
        return extensions
            .iter()
            .map(|(key, value)| {
                let value = match value {
                    serde_json::Value::String(value) => value.clone(),
                    serde_json::Value::Bool(value) => value.to_string(),
                    serde_json::Value::Number(value) => value.to_string(),
                    _ => {
                        return Err(format!(
                            "Hysteria2 option {key:?} must be a string, boolean, or number"
                        ));
                    }
                };
                Ok((key.clone(), value))
            })
            .collect();
    }

    let parsed = Url::parse(&node.raw_config).map_err(|error| error.to_string())?;
    Ok(parsed
        .query_pairs()
        .filter(|(key, _)| key != "sni")
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect())
}

fn apply_insecure(outbound: &mut serde_json::Value, value: &str) -> Result<(), String> {
    match value {
        "1" | "true" => outbound["tls"]["insecure"] = serde_json::json!(true),
        "0" | "false" => {}
        _ => {
            return Err(format!(
                "Hysteria2 insecure must be true, false, 1, or 0; got {value:?}"
            ));
        }
    }
    Ok(())
}

fn apply_alpn(outbound: &mut serde_json::Value, value: &str) -> Result<(), String> {
    let alpn = split_csv(value);
    if alpn.is_empty() {
        return Err("Hysteria2 alpn must contain at least one protocol".to_string());
    }
    outbound["tls"]["alpn"] = serde_json::json!(alpn);
    Ok(())
}

fn apply_obfs(outbound: &mut serde_json::Value, value: &str) -> Result<(), String> {
    if value != "salamander" {
        return Err(format!(
            "Hysteria2 obfs {value:?} is unsupported by sing-box 1.13; only salamander is supported"
        ));
    }
    outbound["obfs"] = serde_json::json!({"type": "salamander"});
    Ok(())
}

fn apply_obfs_password(
    outbound: &mut serde_json::Value,
    value: &str,
    has_obfs: bool,
) -> Result<(), String> {
    if !has_obfs {
        return Err("Hysteria2 obfs-password requires obfs=salamander".to_string());
    }
    if value.is_empty() {
        return Err("Hysteria2 obfs-password must not be empty".to_string());
    }
    outbound["obfs"]["password"] = serde_json::json!(value);
    Ok(())
}

fn apply_mbps(outbound: &mut serde_json::Value, field: &str, value: &str) -> Result<(), String> {
    let parsed = value.parse::<u32>().map_err(|_| {
        format!("Hysteria2 {field} must be an unsigned Mbps integer; got {value:?}")
    })?;
    outbound[field] = serde_json::json!(parsed);
    Ok(())
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}
