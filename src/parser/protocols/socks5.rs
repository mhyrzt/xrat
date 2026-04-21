use url::Url;

use crate::model::{Node, Protocol};

use super::super::support::{empty_to_none, password_or_none, percent_decode, username_or_none};

pub fn parse_socks5(line: &str) -> Result<Node, Box<dyn std::error::Error>> {
    let parsed = Url::parse(line)?;
    let address = parsed
        .host_str()
        .ok_or("missing address or port")?
        .to_string();
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
        name: parsed
            .fragment()
            .map(percent_decode)
            .and_then(empty_to_none),
    })
}
