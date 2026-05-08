use crate::model::Node;

use super::protocols;

pub fn parse_line(line: &str) -> Option<Node> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }

    let parsed = match line {
        value if value.starts_with("vless://") => protocols::parse_vless(value),
        value if value.starts_with("vmess://") => protocols::parse_vmess(value),
        value if value.starts_with("ss://") => protocols::parse_ss(value),
        value if value.starts_with("trojan://") => protocols::parse_trojan(value),
        value if value.starts_with("http://") || value.starts_with("https://") => {
            protocols::parse_http(value)
        }
        value if value.starts_with("socks5://") => protocols::parse_socks5(value),
        value if value.starts_with("hysteria2://") || value.starts_with("hy2://") => {
            protocols::parse_hy2(value)
        }
        _ => return None,
    };

    match parsed {
        Ok(node) => Some(node),
        Err(err) => {
            let preview: String = line.chars().take(80).collect();
            tracing::warn!(%preview, error = %err, "failed to parse config line");
            None
        }
    }
}
