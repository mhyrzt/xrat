use super::super::parse_text;
use crate::model::Protocol;

#[test]
fn keeps_nodes_with_different_runtime_settings() {
    let input = concat!(
        "vless://uuid-123@example.com:443?type=tcp#One\n",
        "vless://uuid-123@example.com:443?type=ws&sni=cdn.example.com#Two\n"
    );

    let nodes = parse_text(input);
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].name.as_deref(), Some("One"));
    assert_eq!(nodes[1].name.as_deref(), Some("Two"));
}

#[test]
fn deduplicates_when_only_display_name_changes() {
    let input = concat!(
        "vless://uuid-123@example.com:443?type=ws&sni=cdn.example.com&path=%2Fray#One\n",
        "vless://uuid-123@example.com:443?type=ws&sni=cdn.example.com&path=%2Fray#Two\n"
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
