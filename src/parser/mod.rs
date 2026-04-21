mod line;
mod normalize;
mod protocols;
mod support;

use std::collections::HashSet;

use crate::model::Node;

pub fn parse_text(config_text: &str) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut seen = HashSet::new();

    for line in config_text.lines() {
        let Some(mut node) = line::parse_line(line) else {
            continue;
        };

        normalize::normalize(&mut node);
        if seen.insert(node.dedup_key()) {
            nodes.push(node);
        }
    }

    nodes
}

#[cfg(test)]
mod tests {
    use super::parse_text;
    use crate::model::Protocol;

    #[test]
    fn parses_vless_like_python_reference() {
        let input = "vless://uuid-123@example.com:443?type=ws&security=tls&sni=cdn.example.com&path=%2Fsocket#Example%20Node";
        let nodes = parse_text(input);

        assert_eq!(nodes.len(), 1);
        let node = &nodes[0];
        assert_eq!(node.protocol, Protocol::Vless);
        assert_eq!(node.address, "example.com");
        assert_eq!(node.port, 443);
        assert_eq!(node.uuid.as_deref(), Some("uuid-123"));
        assert_eq!(node.network, "ws");
        assert_eq!(node.tls.as_deref(), Some("tls"));
        assert_eq!(node.sni.as_deref(), Some("cdn.example.com"));
        assert_eq!(node.host.as_deref(), Some("cdn.example.com"));
        assert_eq!(node.path.as_deref(), Some("/socket"));
        assert_eq!(node.name.as_deref(), Some("Example Node"));
    }

    #[test]
    fn parses_vmess_like_python_reference() {
        let input = "vmess://eyJhZGQiOiJ2bWVzcy5leGFtcGxlLmNvbSIsInBvcnQiOiI4NDQzIiwiaWQiOiJ1dWlkLTQ1NiIsIm5ldCI6IndzIiwidGxzIjoidGxzIiwic25pIjoiZWRnZS5leGFtcGxlLmNvbSIsImhvc3QiOiJob3N0LmV4YW1wbGUuY29tIiwicGF0aCI6Ii92bWVzcyIsInBzIjoiVk1lc3MgTm9kZSJ9";
        let nodes = parse_text(input);

        assert_eq!(nodes.len(), 1);
        let node = &nodes[0];
        assert_eq!(node.protocol, Protocol::Vmess);
        assert_eq!(node.address, "vmess.example.com");
        assert_eq!(node.port, 8443);
        assert_eq!(node.uuid.as_deref(), Some("uuid-456"));
        assert_eq!(node.network, "ws");
        assert_eq!(node.tls.as_deref(), Some("tls"));
        assert_eq!(node.sni.as_deref(), Some("edge.example.com"));
        assert_eq!(node.host.as_deref(), Some("host.example.com"));
        assert_eq!(node.path.as_deref(), Some("/vmess"));
        assert_eq!(node.name.as_deref(), Some("VMess Node"));
    }

    #[test]
    fn parses_ss_like_python_reference() {
        let input = "ss://YWVzLTI1Ni1nY206c2VjcmV0@example.com:8388#SS%20Node";
        let nodes = parse_text(input);

        assert_eq!(nodes.len(), 1);
        let node = &nodes[0];
        assert_eq!(node.protocol, Protocol::Ss);
        assert_eq!(node.address, "example.com");
        assert_eq!(node.port, 8388);
        assert_eq!(node.method.as_deref(), Some("aes-256-gcm"));
        assert_eq!(node.password.as_deref(), Some("secret"));
        assert_eq!(node.network, "tcp");
        assert_eq!(node.name.as_deref(), Some("SS Node"));
    }

    #[test]
    fn normalizes_ws_host_and_path() {
        let input =
            "vless://uuid-123@example.com:443?type=ws&security=tls&sni=cdn.example.com#Node";
        let nodes = parse_text(input);

        assert_eq!(nodes.len(), 1);
        let node = &nodes[0];
        assert_eq!(node.host.as_deref(), Some("cdn.example.com"));
        assert_eq!(node.path.as_deref(), Some("/"));
    }

    #[test]
    fn normalizes_grpc_path_and_empty_tls() {
        let input = "vless://uuid-123@example.com:443?type=grpc&security=#Node";
        let nodes = parse_text(input);

        assert_eq!(nodes.len(), 1);
        let node = &nodes[0];
        assert_eq!(node.network, "grpc");
        assert_eq!(node.path.as_deref(), Some("/"));
        assert_eq!(node.tls, None);
    }

    #[test]
    fn deduplicates_by_shared_key() {
        let input = concat!(
            "vless://uuid-123@example.com:443?type=tcp#One\n",
            "vless://uuid-123@example.com:443?type=ws&sni=cdn.example.com#Two\n"
        );

        let nodes = parse_text(input);

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].name.as_deref(), Some("One"));
    }

    #[test]
    fn skips_comments_blank_lines_and_unknown_protocols() {
        let input = concat!(
            "# comment\n",
            "\n",
            "unknown://ignored\n",
            "ss://YWVzLTI1Ni1nY206c2VjcmV0@example.com:8388#SS%20Node\n"
        );

        let nodes = parse_text(input);

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].protocol, Protocol::Ss);
    }
}
