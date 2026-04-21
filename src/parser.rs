use std::collections::HashSet;

use percent_encoding::percent_decode_str;
use url::{Url, form_urlencoded};

use crate::decode::b64_decode_text;
use crate::model::{Node, Protocol};

pub fn parse_text(config_text: &str) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut seen = HashSet::new();

    for line in config_text.lines() {
        let Some(mut node) = parse_line(line) else {
            continue;
        };

        normalize(&mut node);
        if seen.insert(node.dedup_key()) {
            nodes.push(node);
        }
    }

    nodes
}

fn parse_line(line: &str) -> Option<Node> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }

    let parsed = match line {
        value if value.starts_with("vless://") => parse_vless(value),
        value if value.starts_with("vmess://") => parse_vmess(value),
        value if value.starts_with("ss://") => parse_ss(value),
        value if value.starts_with("trojan://") => parse_trojan(value),
        value if value.starts_with("http://") || value.starts_with("https://") => parse_http(value),
        value if value.starts_with("socks5://") => parse_socks5(value),
        _ => return None,
    };

    match parsed {
        Ok(node) => Some(node),
        Err(err) => {
            let preview: String = line.chars().take(80).collect();
            eprintln!("[ERROR] Failed to parse line: {preview} ... Reason: {err}");
            None
        }
    }
}

fn parse_vless(line: &str) -> Result<Node, Box<dyn std::error::Error>> {
    let parsed = Url::parse(&line.replacen("vless://", "https://", 1))?;
    let address = parsed.host_str().ok_or("missing address or port")?.to_string();
    let port = parsed.port().ok_or("missing address or port")?;
    let query = parse_query_pairs(parsed.query().unwrap_or_default());
    let fragment = parsed.fragment().map(percent_decode);
    let path = query
        .get("path")
        .map(String::as_str)
        .unwrap_or_default();

    Ok(Node {
        protocol: Protocol::Vless,
        address,
        port,
        username: None,
        uuid: username_or_none(&parsed),
        password: None,
        method: None,
        network: query
            .get("type")
            .cloned()
            .unwrap_or_else(|| "tcp".to_string()),
        tls: query.get("security").cloned(),
        sni: query.get("sni").cloned(),
        host: query.get("host").cloned(),
        path: empty_to_none(percent_decode(path)),
        name: fragment.and_then(empty_to_none),
    })
}

fn parse_vmess(line: &str) -> Result<Node, Box<dyn std::error::Error>> {
    let payload = line.trim_start_matches("vmess://");
    let data: serde_json::Value = serde_json::from_str(&b64_decode_text(payload)?)?;

    let address = required_string(&data, "add")?;
    let port: u16 = required_string(&data, "port")?.parse()?;

    Ok(Node {
        protocol: Protocol::Vmess,
        address,
        port,
        username: None,
        uuid: optional_string(&data, "id"),
        password: None,
        method: None,
        network: optional_string(&data, "net").unwrap_or_else(|| "tcp".to_string()),
        tls: optional_string(&data, "tls"),
        sni: optional_string(&data, "sni"),
        host: optional_string(&data, "host"),
        path: optional_string(&data, "path"),
        name: optional_string(&data, "ps"),
    })
}

fn parse_ss(line: &str) -> Result<Node, Box<dyn std::error::Error>> {
    let parsed = Url::parse(&line.replacen("ss://", "https://", 1))?;
    let address = parsed.host_str().ok_or("missing address or port")?.to_string();
    let port = parsed.port().ok_or("missing address or port")?;
    let userinfo = parsed.username();
    if userinfo.is_empty() {
        return Err("missing base64 userinfo".into());
    }

    let decoded = b64_decode_text(userinfo)?;
    let (method, password) = decoded
        .split_once(':')
        .ok_or("invalid Shadowsocks userinfo format")?;

    Ok(Node {
        protocol: Protocol::Ss,
        address,
        port,
        username: None,
        uuid: None,
        password: Some(password.to_string()),
        method: Some(method.to_string()),
        network: "tcp".to_string(),
        tls: None,
        sni: None,
        host: None,
        path: None,
        name: parsed.fragment().map(percent_decode).and_then(empty_to_none),
    })
}

fn parse_trojan(line: &str) -> Result<Node, Box<dyn std::error::Error>> {
    let parsed = Url::parse(&line.replacen("trojan://", "https://", 1))?;
    let address = parsed.host_str().ok_or("missing address or port")?.to_string();
    let port = parsed.port().ok_or("missing address or port")?;
    let query = parse_query_pairs(parsed.query().unwrap_or_default());
    let fragment = parsed.fragment().map(percent_decode);
    let path = query
        .get("path")
        .map(String::as_str)
        .unwrap_or_default();

    Ok(Node {
        protocol: Protocol::Trojan,
        address,
        port,
        username: None,
        uuid: None,
        password: username_or_none(&parsed),
        method: None,
        network: query
            .get("type")
            .cloned()
            .unwrap_or_else(|| "tcp".to_string()),
        tls: query
            .get("security")
            .cloned()
            .or_else(|| Some("tls".to_string())),
        sni: query.get("sni").cloned(),
        host: query.get("host").cloned(),
        path: empty_to_none(percent_decode(path)),
        name: fragment.and_then(empty_to_none),
    })
}

fn parse_http(line: &str) -> Result<Node, Box<dyn std::error::Error>> {
    let parsed = Url::parse(line)?;
    let address = parsed.host_str().ok_or("missing address or port")?.to_string();
    let port = parsed
        .port_or_known_default()
        .ok_or("missing address or port")?;

    Ok(Node {
        protocol: Protocol::Http,
        address,
        port,
        username: username_or_none(&parsed),
        uuid: None,
        password: password_or_none(&parsed),
        method: None,
        network: "tcp".to_string(),
        tls: (parsed.scheme() == "https").then(|| "tls".to_string()),
        sni: None,
        host: None,
        path: None,
        name: parsed.fragment().map(percent_decode).and_then(empty_to_none),
    })
}

fn parse_socks5(line: &str) -> Result<Node, Box<dyn std::error::Error>> {
    let parsed = Url::parse(&line.replacen("socks5://", "https://", 1))?;
    let address = parsed.host_str().ok_or("missing address or port")?.to_string();
    let port = parsed.port().ok_or("missing address or port")?;

    Ok(Node {
        protocol: Protocol::Socks5,
        address,
        port,
        username: username_or_none(&parsed),
        uuid: None,
        password: password_or_none(&parsed),
        method: None,
        network: "tcp".to_string(),
        tls: None,
        sni: None,
        host: None,
        path: None,
        name: parsed.fragment().map(percent_decode).and_then(empty_to_none),
    })
}

fn normalize(node: &mut Node) {
    if node.network.is_empty() {
        node.network = "tcp".to_string();
    }

    if node.network == "ws" {
        if node.host.is_none() {
            node.host = node.sni.clone();
        }
        if node.path.is_none() {
            node.path = Some("/".to_string());
        }
    }

    if node.network == "grpc" && node.path.is_none() {
        node.path = Some("/".to_string());
    }

    if matches!(node.tls.as_deref(), Some("")) {
        node.tls = None;
    }
}

fn parse_query_pairs(query: &str) -> std::collections::HashMap<String, String> {
    form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect()
}

fn required_string(
    value: &serde_json::Value,
    key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    optional_string(value, key)
        .ok_or_else(|| format!("missing required {key} field in vmess JSON").into())
}

fn optional_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value.get(key).and_then(|item| item.as_str()).map(ToOwned::to_owned)
}

fn username_or_none(url: &Url) -> Option<String> {
    if url.username().is_empty() {
        None
    } else {
        Some(url.username().to_string())
    }
}

fn password_or_none(url: &Url) -> Option<String> {
    url.password().map(ToOwned::to_owned)
}

fn percent_decode(value: &str) -> String {
    percent_decode_str(value).decode_utf8_lossy().into_owned()
}

fn empty_to_none(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}
