use super::super::parse_text;
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
fn captures_vless_xhttp_extensions() {
    let input = "vless://uuid-123@example.com:2087?type=xhttp&security=tls&host=cdn.example.com&mode=auto&sni=cdn.example.com&fp=chrome&alpn=h2#Node";
    let nodes = parse_text(input);
    assert_eq!(nodes.len(), 1);
    let node = &nodes[0];
    assert_eq!(node.network, "xhttp");
    let extensions = node.extensions.as_ref().unwrap();
    assert_eq!(extensions.get("fp").map(String::as_str), Some("chrome"));
    assert_eq!(extensions.get("alpn").map(String::as_str), Some("h2"));
    assert_eq!(extensions.get("mode").map(String::as_str), Some("auto"));
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
    let input = "vless://uuid-123@example.com:443?type=ws&security=tls&sni=cdn.example.com#Node";
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
fn parses_hy2_line() {
    let input =
        "hy2://secret@example.com:443?sni=edge.example.com&obfs=salamander&obfs-password=123#HY2";
    let nodes = parse_text(input);
    assert_eq!(nodes.len(), 1);
    let node = &nodes[0];
    assert_eq!(node.protocol, Protocol::Hy2);
    assert_eq!(node.password.as_deref(), Some("secret"));
    assert_eq!(node.sni.as_deref(), Some("edge.example.com"));
}

#[test]
fn parses_hysteria2_alias_line() {
    let input = "hysteria2://secret@example.com:8443?sni=edge.example.com#Node";
    let nodes = parse_text(input);

    assert_eq!(nodes.len(), 1);
    let node = &nodes[0];
    assert_eq!(node.protocol, Protocol::Hy2);
    assert_eq!(node.port, 8443);
    assert_eq!(node.name.as_deref(), Some("Node"));
}

#[test]
fn normalizes_hy2_defaults() {
    let input = "hy2://secret@example.com:443#";
    let nodes = parse_text(input);

    assert_eq!(nodes.len(), 1);
    let node = &nodes[0];
    assert_eq!(node.protocol, Protocol::Hy2);
    assert_eq!(node.network, "udp");
    assert_eq!(node.tls.as_deref(), Some("tls"));
    assert_eq!(node.name, None);
}
