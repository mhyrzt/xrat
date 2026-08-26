use percent_encoding::percent_decode_str;
use std::collections::{BTreeMap, HashSet};
use url::{Url, form_urlencoded};

use super::ConfigParseError;

pub fn parse_query_pairs(query: &str) -> std::collections::HashMap<String, String> {
    form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect()
}

pub fn reject_duplicate_query_parameters(query: &str) -> Result<(), ConfigParseError> {
    let mut seen = HashSet::new();
    for (key, _) in form_urlencoded::parse(query.as_bytes()) {
        let key = key.into_owned();
        if !seen.insert(key.clone()) {
            return Err(ConfigParseError::DuplicateQueryParameter(key));
        }
    }
    Ok(())
}

pub fn query_extensions(
    query: &str,
    structural_keys: &[&str],
) -> Option<BTreeMap<String, serde_json::Value>> {
    let structural = structural_keys.iter().copied().collect::<HashSet<_>>();
    let mut extensions = BTreeMap::new();
    for (key, value) in form_urlencoded::parse(query.as_bytes()).into_owned() {
        if structural.contains(key.as_str()) {
            continue;
        }
        match extensions.remove(&key) {
            None => {
                extensions.insert(key, serde_json::Value::String(value));
            }
            Some(serde_json::Value::Array(mut values)) => {
                values.push(serde_json::Value::String(value));
                extensions.insert(key, serde_json::Value::Array(values));
            }
            Some(previous) => {
                extensions.insert(
                    key,
                    serde_json::Value::Array(vec![previous, serde_json::Value::String(value)]),
                );
            }
        }
    }
    (!extensions.is_empty()).then_some(extensions)
}

pub fn required_string(
    value: &serde_json::Value,
    key: &'static str,
) -> Result<String, ConfigParseError> {
    optional_string(value, key).ok_or(ConfigParseError::MissingRequiredField {
        context: "vmess JSON",
        key,
    })
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
        Some(percent_decode(url.username()))
    }
}

pub fn password_or_none(url: &Url) -> Option<String> {
    url.password().map(percent_decode)
}

pub fn percent_decode(value: &str) -> String {
    percent_decode_str(value).decode_utf8_lossy().into_owned()
}

pub fn empty_to_none(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}
