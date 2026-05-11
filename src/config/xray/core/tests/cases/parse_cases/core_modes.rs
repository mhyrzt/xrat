use super::*;

#[test]
fn test_parse_minimal_config_loose() {
    let json = r#"{
        "inbounds": [],
        "outbounds": []
    }"#;

    let config = XrayConfig::from_json_loose(json).unwrap();
    assert!(config.inbounds.is_some());
    assert!(config.outbounds.is_some());
    assert!(config.log.is_none());
}

#[test]
fn test_parse_minimal_config_strict() {
    let json = r#"{
        "inbounds": [],
        "outbounds": []
    }"#;

    let config = XrayConfig::from_json_strict(json).unwrap();
    assert!(config.inbounds.is_some());
    assert!(config.outbounds.is_some());
}

#[test]
fn test_loose_mode_allows_unknown_fields() {
    let json = r#"{
        "inbounds": [],
        "outbounds": [],
        "unknownField": "should be ignored",
        "anotherUnknown": 123
    }"#;

    let result = XrayConfig::from_json_loose(json);
    assert!(result.is_ok(), "Loose mode should allow unknown fields");
}

#[test]
fn test_strict_mode_rejects_unknown_fields() {
    let json = r#"{
        "inbounds": [],
        "outbounds": [],
        "unknownField": "should cause error"
    }"#;

    let result = XrayConfig::from_json_strict(json);
    assert!(result.is_err(), "Strict mode should reject unknown fields");
}

#[test]
fn test_parse_mode_controls_unknown_fields() {
    let json = r#"{
        "inbounds": [],
        "outbounds": [],
        "unknownField": "allowed outside strict mode"
    }"#;

    assert!(XrayConfig::from_json_with_mode(json, ParseMode::Strict).is_err());
    assert!(XrayConfig::from_json_with_mode(json, ParseMode::Lenient).is_ok());
    assert!(XrayConfig::from_json_with_mode(json, ParseMode::Auto).is_ok());
}
