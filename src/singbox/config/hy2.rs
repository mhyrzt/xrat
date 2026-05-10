use url::Url;

use crate::model::Node;

pub fn build_hy2_outbound(node: &Node) -> Result<serde_json::Value, String> {
    let mut outbound = serde_json::json!({
        "type": "hysteria2",
        "tag": "proxy",
        "server": node.address,
        "server_port": node.port,
        "password": node.password.as_deref().unwrap_or_default(),
        "tls": {
            "enabled": true,
            "server_name": node.sni.as_deref().unwrap_or(&node.address)
        }
    });

    let options = if let Some(extensions) = &node.extensions {
        extensions.clone()
    } else {
        let parsed = Url::parse(&node.raw_config).map_err(|error| error.to_string())?;
        parsed
            .query_pairs()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    };

    for (key, value) in options {
        match key.as_str() {
            "insecure" => apply_insecure(&mut outbound, &value),
            "alpn" => apply_alpn(&mut outbound, &value),
            "obfs" => apply_obfs(&mut outbound, &value),
            "obfs-password" => apply_obfs_password(&mut outbound, &value),
            "upmbps" => apply_mbps(&mut outbound, "up_mbps", &value),
            "downmbps" => apply_mbps(&mut outbound, "down_mbps", &value),
            _ => {}
        }
    }

    Ok(outbound)
}

fn apply_insecure(outbound: &mut serde_json::Value, value: &str) {
    if matches!(value, "1" | "true") {
        outbound["tls"]["insecure"] = serde_json::json!(true);
    }
}

fn apply_alpn(outbound: &mut serde_json::Value, value: &str) {
    let alpn = split_csv(value);
    if !alpn.is_empty() {
        outbound["tls"]["alpn"] = serde_json::json!(alpn);
    }
}

fn apply_obfs(outbound: &mut serde_json::Value, value: &str) {
    if value == "salamander" {
        outbound["obfs"] = serde_json::json!({"type": "salamander"});
    }
}

fn apply_obfs_password(outbound: &mut serde_json::Value, value: &str) {
    if value.is_empty() {
        return;
    }
    if outbound.get("obfs").is_none() {
        outbound["obfs"] = serde_json::json!({"type": "salamander"});
    }
    outbound["obfs"]["password"] = serde_json::json!(value);
}

fn apply_mbps(outbound: &mut serde_json::Value, field: &str, value: &str) {
    if let Ok(parsed) = value.parse::<u32>() {
        outbound[field] = serde_json::json!(parsed);
    }
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}
