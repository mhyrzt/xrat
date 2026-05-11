use super::super::{ImportParseError, ImportResult};
use crate::model::Node;

pub fn parse_sip008_json(input: &str) -> Result<ImportResult, ImportParseError> {
    use serde_json::Value;

    let json: Value = serde_json::from_str(input)?;
    let servers = json
        .get("servers")
        .and_then(|v| v.as_array())
        .ok_or(ImportParseError::MissingSip008Servers)?;

    let mut nodes = Vec::new();
    let mut errors = Vec::new();
    for (idx, server) in servers.iter().enumerate() {
        match parse_sip008_server(server) {
            Ok(node) => nodes.push(node),
            Err(error) => errors.push((idx + 1, format!("Server {}: {}", idx + 1, error))),
        }
    }

    Ok(ImportResult {
        nodes,
        errors,
        metadata: None,
    })
}

fn parse_sip008_server(server: &serde_json::Value) -> Result<Node, ImportParseError> {
    use crate::model::Protocol;

    let address = server
        .get("server")
        .and_then(|v| v.as_str())
        .ok_or(ImportParseError::MissingSip008Field("server"))?
        .to_string();
    let port = server
        .get("server_port")
        .and_then(|v| v.as_u64())
        .ok_or(ImportParseError::MissingSip008Field("server_port"))? as u16;
    let method = server
        .get("method")
        .and_then(|v| v.as_str())
        .ok_or(ImportParseError::MissingSip008Field("method"))?
        .to_string();
    let password = server
        .get("password")
        .and_then(|v| v.as_str())
        .ok_or(ImportParseError::MissingSip008Field("password"))?
        .to_string();
    let name = server
        .get("remarks")
        .or_else(|| server.get("name"))
        .and_then(|v| v.as_str())
        .map(str::to_string);

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
        extensions: None,
        raw_config: serde_json::to_string(server)?,
    })
}
