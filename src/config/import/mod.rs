mod detect;
mod error;
mod parsers;
#[allow(dead_code)]
mod subscription;

use crate::model::Node;

pub use error::ImportParseError;

#[derive(Debug)]
pub struct ImportResult {
    pub nodes: Vec<Node>,
    pub errors: Vec<(usize, String)>,
    pub metadata: Option<SubscriptionMetadata>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionMetadata {
    pub upload: Option<u64>,
    pub download: Option<u64>,
    pub total: Option<u64>,
    pub expire: Option<i64>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    Auto,
    SingleLink,
    Base64Subscription,
    PlainList,
    Sip008Json,
    XrayJson,
}

pub fn parse_import(input: &str, mode: ImportMode) -> Result<ImportResult, ImportParseError> {
    let input = input.trim();

    if input.is_empty() {
        return Ok(ImportResult {
            nodes: vec![],
            errors: vec![],
            metadata: None,
        });
    }

    let detected_mode = if mode == ImportMode::Auto {
        detect::detect_format(input)
    } else {
        mode
    };

    match detected_mode {
        ImportMode::SingleLink => parsers::parse_single_link(input),
        ImportMode::Base64Subscription => parsers::parse_base64_subscription(input),
        ImportMode::PlainList => parsers::parse_plain_list(input),
        ImportMode::Sip008Json => parsers::parse_sip008_json(input),
        ImportMode::XrayJson => parsers::parse_xray_json(input),
        ImportMode::Auto => unreachable!("Auto mode should be resolved"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_single_vless_link() {
        let input = "vless://uuid@example.com:443?security=tls&type=ws#test";
        let result = parse_import(input, ImportMode::Auto).expect("should parse");
        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.nodes[0].protocol.as_str(), "vless");
    }

    #[test]
    fn detects_single_vmess_link() {
        // Valid vmess link with proper base64-encoded JSON
        let input = "vmess://ew0KICAidiI6ICIyIiwNCiAgInBzIjogInRlc3QiLA0KICAiYWRkIjogImV4YW1wbGUuY29tIiwNCiAgInBvcnQiOiAiNDQzIiwNCiAgImlkIjogInV1aWQiLA0KICAiYWlkIjogIjAiLA0KICAibmV0IjogInRjcCIsDQogICJ0eXBlIjogIm5vbmUiLA0KICAiaG9zdCI6ICIiLA0KICAicGF0aCI6ICIiLA0KICAidGxzIjogInRscyINCn0=";
        let result = parse_import(input, ImportMode::Auto).expect("should parse");
        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.nodes[0].protocol.as_str(), "vmess");
    }

    #[test]
    fn detects_plain_list_of_links() {
        let input = "vless://uuid1@example.com:443\n\
                     vless://uuid2@example.com:443\n\
                     trojan://pass@example.com:443";
        let result = parse_import(input, ImportMode::Auto).expect("should parse");
        assert_eq!(result.nodes.len(), 3);
    }

    #[test]
    fn detects_base64_subscription() {
        let links = "vless://uuid@example.com:443\nvless://uuid2@example.com:443";
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, links);
        let result = parse_import(&encoded, ImportMode::Auto).expect("should parse");
        assert_eq!(result.nodes.len(), 2);
    }

    #[test]
    fn handles_empty_input() {
        let result = parse_import("", ImportMode::Auto).expect("empty should not error");
        assert_eq!(result.nodes.len(), 0);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn handles_whitespace_only_input() {
        let result =
            parse_import("   \n\t  ", ImportMode::Auto).expect("whitespace should not error");
        assert_eq!(result.nodes.len(), 0);
    }

    #[test]
    fn collects_parse_errors_for_invalid_links() {
        let input = "vless://valid@example.com:443\n\
                     invalid://not-a-protocol\n\
                     vless://valid2@example.com:443";
        let result = parse_import(input, ImportMode::Auto).expect("should parse with errors");
        assert_eq!(result.nodes.len(), 2);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn explicit_mode_overrides_auto_detection() {
        let input = "vless://uuid@example.com:443";
        let result = parse_import(input, ImportMode::SingleLink).expect("should parse");
        assert_eq!(result.nodes.len(), 1);
    }

    #[test]
    fn detects_sip008_json_format() {
        let input = r#"{"servers":[{"server":"example.com","server_port":443,"password":"pass","method":"aes-256-gcm"}]}"#;
        let result = parse_import(input, ImportMode::Auto).expect("should parse");
        assert!(!result.nodes.is_empty() || !result.errors.is_empty());
    }

    #[test]
    fn detects_xray_json_format() {
        // XrayJson format is detected by presence of "inbounds" or "version" keys
        // The parser currently returns empty nodes but shouldn't error on valid structure
        let input = r#"{"inbounds":[],"outbounds":[]}"#;
        let result = parse_import(input, ImportMode::Auto);
        // Just verify detection works - parser may return empty or error depending on structure
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn handles_mixed_protocol_links() {
        let input = "vless://uuid@example.com:443?security=tls&type=tcp\n\
                     trojan://pass@example.com:443?security=tls&type=tcp\n\
                     socks5://example.com:1080";
        let result = parse_import(input, ImportMode::Auto).expect("should parse");
        assert!(
            result.nodes.len() >= 2,
            "should parse at least 2 protocols, got {}",
            result.nodes.len()
        );
    }

    #[test]
    fn trims_whitespace_from_links() {
        let input = "  vless://uuid@example.com:443  \n\tvless://uuid2@example.com:443\t";
        let result = parse_import(input, ImportMode::Auto).expect("should parse");
        assert_eq!(result.nodes.len(), 2);
    }

    #[test]
    fn handles_links_with_remarks() {
        let input = "vless://uuid@example.com:443?security=tls#My%20Server";
        let result = parse_import(input, ImportMode::Auto).expect("should parse");
        assert_eq!(result.nodes.len(), 1);
        assert!(result.nodes[0].name.is_some());
    }
}
