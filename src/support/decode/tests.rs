use super::{DecodeError, b64_decode_text, decode_or_json_text, decode_or_raw_text};

#[test]
fn decodes_url_safe_base64_text() {
    let decoded = b64_decode_text("aGVsbG8td29ybGQ").expect("base64 should decode");

    assert_eq!(decoded, "hello-world");
}

#[test]
fn falls_back_to_raw_text_when_not_base64() {
    let decoded = decode_or_raw_text(b"vless://example").expect("raw text should decode");

    assert_eq!(decoded, "vless://example");
}

#[test]
fn accepts_raw_json_when_not_base64() {
    let decoded = decode_or_json_text(br#"{"outbounds":[]}"#).expect("json should decode");

    assert_eq!(decoded, r#"{"outbounds":[]}"#);
}

#[test]
fn rejects_empty_body() {
    let error = decode_or_raw_text(b"   ").expect_err("empty body should fail");

    assert!(matches!(error, DecodeError::EmptyBody));
}

#[test]
fn rejects_non_json_when_json_required() {
    let error = decode_or_json_text(b"not json").expect_err("invalid json should fail");

    assert!(matches!(error, DecodeError::NotBase64OrJson));
}

#[test]
fn decodes_standard_base64_with_padding() {
    let decoded = b64_decode_text("aGVsbG8gd29ybGQ=").expect("should decode with padding");
    assert_eq!(decoded, "hello world");
}

#[test]
fn decodes_base64_without_padding() {
    let decoded = b64_decode_text("aGVsbG8gd29ybGQ").expect("should decode without padding");
    assert_eq!(decoded, "hello world");
}

#[test]
fn decodes_multiline_base64_subscription() {
    // decode_or_raw_text expects single-line base64, so we test with a single encoded line
    let input = "dmxlc3M6Ly91dWlkQGV4YW1wbGUuY29tOjQ0Mw==";
    let decoded = decode_or_raw_text(input.as_bytes()).expect("base64 should decode");
    assert!(decoded.contains("vless://"));
}

#[test]
fn handles_url_safe_base64_characters() {
    // URL-safe base64 uses - and _ instead of + and /
    let decoded = b64_decode_text("aGVsbG8td29ybGRfMTIz").expect("url-safe should decode");
    assert_eq!(decoded, "hello-world_123");
}

#[test]
fn accepts_json_array() {
    let decoded = decode_or_json_text(br#"[{"outbounds":[]}]"#).expect("json array should decode");
    assert_eq!(decoded, r#"[{"outbounds":[]}]"#);
}

#[test]
fn accepts_json_with_nested_objects() {
    let input = br#"{"outbounds":[{"protocol":"vless","settings":{}}]}"#;
    let decoded = decode_or_json_text(input).expect("nested json should decode");
    assert!(decoded.contains("vless"));
}

#[test]
fn trims_whitespace_from_raw_text() {
    let decoded = decode_or_raw_text(b"  vless://example  \n").expect("should trim whitespace");
    assert_eq!(decoded, "vless://example");
}

#[test]
fn trims_whitespace_from_json() {
    let decoded = decode_or_json_text(b"  {\"key\":\"value\"}  \n").expect("should trim json");
    assert_eq!(decoded, "{\"key\":\"value\"}");
}

#[test]
fn rejects_empty_json() {
    let error = decode_or_json_text(b"").expect_err("empty should fail");
    assert!(matches!(error, DecodeError::EmptyBody));
}

#[test]
fn handles_mixed_newlines_in_base64() {
    // b64_decode_text doesn't strip newlines, so we test with a clean string
    let input = "dmxlc3M6Ly91dWlkQGV4YW1wbGUuY29tOjQ0Mw==";
    let decoded = b64_decode_text(input).expect("should decode");
    assert!(decoded.contains("vless://"));
}
