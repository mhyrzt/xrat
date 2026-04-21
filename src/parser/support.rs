use percent_encoding::percent_decode_str;
use url::{Url, form_urlencoded};

pub fn parse_query_pairs(query: &str) -> std::collections::HashMap<String, String> {
    form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect()
}

pub fn required_string(
    value: &serde_json::Value,
    key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    optional_string(value, key)
        .ok_or_else(|| format!("missing required {key} field in vmess JSON").into())
}

pub fn optional_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|item| item.as_str())
        .map(ToOwned::to_owned)
}

pub fn username_or_none(url: &Url) -> Option<String> {
    if url.username().is_empty() {
        None
    } else {
        Some(url.username().to_string())
    }
}

pub fn password_or_none(url: &Url) -> Option<String> {
    url.password().map(ToOwned::to_owned)
}

pub fn percent_decode(value: &str) -> String {
    percent_decode_str(value).decode_utf8_lossy().into_owned()
}

pub fn empty_to_none(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}
