use crate::server::parse_bind_addr_public;

#[test]
fn parses_ipv4_and_ipv6_bind_addresses() {
    assert_eq!(
        parse_bind_addr_public("127.0.0.1", 8080)
            .expect("ipv4 should parse")
            .to_string(),
        "127.0.0.1:8080"
    );
    assert_eq!(
        parse_bind_addr_public("::1", 8080)
            .expect("ipv6 should parse")
            .to_string(),
        "[::1]:8080"
    );
}
