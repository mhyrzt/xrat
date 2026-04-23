use super::{ImportResult, SubscriptionMetadata};
use crate::config::xray::XrayConfig;
use crate::model::Node;

pub fn parse_single_link(input: &str) -> Result<ImportResult, Box<dyn std::error::Error>> {
    let node = crate::config::line::parse_line(input).ok_or("Failed to parse share link")?;

    Ok(ImportResult {
        nodes: vec![node],
        errors: vec![],
        metadata: None,
    })
}

pub fn parse_base64_subscription(input: &str) -> Result<ImportResult, Box<dyn std::error::Error>> {
    use crate::support::decode::b64_decode_text;

    let decoded = b64_decode_text(input.trim())?;
    parse_plain_list(&decoded)
}

pub fn parse_plain_list(input: &str) -> Result<ImportResult, Box<dyn std::error::Error>> {
    let mut nodes = Vec::new();
    let mut errors = Vec::new();
    let mut metadata = SubscriptionMetadata {
        upload: None,
        download: None,
        total: None,
        expire: None,
        status: None,
    };

    for (line_num, line) in input.lines().enumerate() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with("STATUS=") {
            metadata.status = Some(line.trim_start_matches("STATUS=").to_string());
            continue;
        }

        match crate::config::line::parse_line(line) {
            Some(mut node) => {
                crate::config::normalize::normalize(&mut node);
                nodes.push(node);
            }
            None => {
                errors.push((
                    line_num + 1,
                    format!(
                        "Failed to parse line: {}",
                        line.chars().take(50).collect::<String>()
                    ),
                ));
            }
        }
    }

    let metadata = if metadata.status.is_some() {
        Some(metadata)
    } else {
        None
    };

    Ok(ImportResult {
        nodes,
        errors,
        metadata,
    })
}

pub fn parse_sip008_json(input: &str) -> Result<ImportResult, Box<dyn std::error::Error>> {
    use serde_json::Value;

    let json: Value = serde_json::from_str(input)?;

    let servers = json
        .get("servers")
        .and_then(|v| v.as_array())
        .ok_or("SIP008 JSON must have 'servers' array")?;

    let mut nodes = Vec::new();
    let mut errors = Vec::new();

    for (idx, server) in servers.iter().enumerate() {
        match parse_sip008_server(server) {
            Ok(node) => nodes.push(node),
            Err(e) => errors.push((idx + 1, format!("Server {}: {}", idx + 1, e))),
        }
    }

    Ok(ImportResult {
        nodes,
        errors,
        metadata: None,
    })
}

fn parse_sip008_server(server: &serde_json::Value) -> Result<Node, Box<dyn std::error::Error>> {
    use crate::model::{Node, Protocol};

    let address = server
        .get("server")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'server' field")?
        .to_string();

    let port = server
        .get("server_port")
        .and_then(|v| v.as_u64())
        .ok_or("Missing 'server_port' field")? as u16;

    let method = server
        .get("method")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'method' field")?
        .to_string();

    let password = server
        .get("password")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'password' field")?
        .to_string();

    let name = server
        .get("remarks")
        .or_else(|| server.get("name"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(Node {
        protocol: Protocol::Ss,
        address,
        port,
        username: None,
        uuid: None,
        password: Some(password),
        method: Some(method),
        network: "tcp".to_string(),
        tls: None,
        sni: None,
        host: None,
        path: None,
        name,
        raw_config: serde_json::to_string(server)?,
    })
}

pub fn parse_xray_json(input: &str) -> Result<ImportResult, Box<dyn std::error::Error>> {
    let _config: XrayConfig = XrayConfig::from_json_loose(input)?;

    Ok(ImportResult {
        nodes: vec![],
        errors: vec![],
        metadata: None,
    })
}
