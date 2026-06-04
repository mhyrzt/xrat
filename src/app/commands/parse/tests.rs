use super::input::{decode_input_text, extract_inputs, parse_inputs, validate_inputs};
use super::json_output::{build_json_value, format_details};
use crate::cli::ParseArgs;
use crate::cli::ParseEngine;
use crate::config::{ResolvedEngine, parse_link};
use std::path::PathBuf;

#[test]
fn validate_inputs_rejects_missing_mode() {
    let args = ParseArgs {
        input: None,
        file: None,
        stdin: false,
        json: false,
        engine: ParseEngine::Auto,
    };
    let error = validate_inputs(&args).expect_err("missing mode should fail");
    assert!(error.to_string().contains("provide one input"));
}

#[test]
fn validate_inputs_rejects_conflicting_modes() {
    let args = ParseArgs {
        input: Some("vless://example".to_string()),
        file: Some(PathBuf::from("/tmp/links.txt")),
        stdin: false,
        json: false,
        engine: ParseEngine::Auto,
    };
    let error = validate_inputs(&args).expect_err("conflict should fail");
    assert!(error.to_string().contains("use only one input mode"));
}

#[test]
fn extract_inputs_tracks_line_context() {
    let parsed = extract_inputs("vless://one\n\n#comment\nvmess://two", "stdin");
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].source, "stdin line 1");
    assert_eq!(parsed[1].source, "stdin line 4");
}

#[test]
fn decode_input_accepts_base64_text_list() {
    let encoded = b"dmxlc3M6Ly9leGFtcGxlLmNvbTo0NDMKc3M6Ly9hYmNAZXhhbXBsZS5jb206ODM4OA==";
    let decoded = decode_input_text(encoded, "stdin").expect("base64 should decode");
    assert!(decoded.contains("vless://example.com:443"));
    assert!(decoded.contains("ss://abc@example.com:8388"));
}

#[test]
fn cleans_null_and_empty_fields_from_json() {
    let link = "vless://uuid-123@example.com:443?type=ws&security=tls&sni=cdn.example.com&path=%2Fray#Node";
    let node = parse_link(link)
        .expect("link should parse")
        .expect("node should exist");
    let value = build_json_value(&node, ResolvedEngine::Xray).expect("json should build");
    assert!(
        !has_null_or_empty(&value),
        "expected cleaned JSON without null/empty fields"
    );
}

fn has_null_or_empty(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => true,
        serde_json::Value::String(text) => text.is_empty(),
        serde_json::Value::Object(map) => map.is_empty() || map.values().any(has_null_or_empty),
        serde_json::Value::Array(values) => {
            values.is_empty() || values.iter().any(has_null_or_empty)
        }
        _ => false,
    }
}

#[test]
fn cleans_empty_array_and_object_values() {
    let value = serde_json::json!({
        "a": null,
        "b": "",
        "c": [],
        "d": {},
        "e": ["ok", null]
    });

    let cleaned =
        super::json_output::clean_json_value(value).expect("should keep non-empty values");
    assert_eq!(cleaned, serde_json::json!({"e": ["ok"]}));
}

#[test]
fn parse_policy_stops_on_first_error() {
    let inputs = vec![
        super::input::ParseInput {
            link: "not-a-url".to_string(),
            source: "stdin line 1".to_string(),
        },
        super::input::ParseInput {
            link: "vless://uuid-123@example.com:443?type=tcp#ok".to_string(),
            source: "stdin line 2".to_string(),
        },
    ];

    let error = parse_inputs(&inputs, ParseEngine::Auto).expect_err("must stop on first bad");
    let rendered = error.to_string();
    assert!(rendered.contains("stdin line 1"));
    assert!(!rendered.contains("stdin line 2"));
}

#[test]
fn formats_details_output_for_valid_link() {
    let link = "vless://uuid-123@example.com:443?type=ws&security=tls&sni=cdn.example.com#Node";
    let node = parse_link(link)
        .expect("link should parse")
        .expect("node should exist");

    let details = format_details(&node, ResolvedEngine::Xray);
    assert!(details.contains("engine"));
    assert!(details.contains("xray"));
    assert!(details.contains("protocol"));
    assert!(details.contains("vless"));
    assert!(details.contains("address"));
    assert!(details.contains("example.com"));
}

#[test]
fn builds_xray_json_for_vless() {
    let link = "vless://uuid-123@example.com:443?type=ws&security=tls&sni=cdn.example.com&path=%2Fray#Node";
    let node = parse_link(link)
        .expect("link should parse")
        .expect("node should exist");
    let value = build_json_value(&node, ResolvedEngine::Xray).expect("json should build");
    assert_eq!(value["outbounds"][0]["protocol"], "vless");
    assert_eq!(value["inbounds"][0]["protocol"], "socks");
}

#[test]
fn builds_singbox_json_for_hy2() {
    let link = "hy2://secret@example.com:443?sni=edge.example.com&obfs=salamander&obfs-password=test&insecure=1&alpn=h3,h2#HY2";
    let node = parse_link(link)
        .expect("link should parse")
        .expect("node should exist");
    let value = build_json_value(&node, ResolvedEngine::SingBox).expect("json should build");
    assert_eq!(value["outbounds"][0]["type"], "hysteria2");
    assert_eq!(value["outbounds"][0]["tls"]["insecure"], true);
}
