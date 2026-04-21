use url::Url;

use crate::model::{Node, Protocol};
use crate::support::decode::b64_decode_text;

use super::super::support::{empty_to_none, percent_decode};

pub fn parse_ss(line: &str) -> Result<Node, Box<dyn std::error::Error>> {
    let parsed = Url::parse(line)?;
    let address = parsed
        .host_str()
        .ok_or("missing address or port")?
        .to_string();
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
        name: parsed
            .fragment()
            .map(percent_decode)
            .and_then(empty_to_none),
    })
}
