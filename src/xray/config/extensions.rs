use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::model::{Node, Protocol};

pub(super) struct ExtensionResolver {
    values: BTreeMap<String, Value>,
    network: String,
}

impl ExtensionResolver {
    pub(super) fn new(node: &Node) -> Self {
        let mut values = node.extensions.clone().unwrap_or_default();
        for key in ["email", "group", "name", "remark", "remarks"] {
            values.remove(key);
        }
        if node.protocol == Protocol::Vmess {
            values.remove("v");
        }
        Self {
            values,
            network: node.network.to_ascii_lowercase(),
        }
    }

    pub(super) fn value(&mut self, key: &str) -> Option<Value> {
        self.values.remove(key)
    }

    pub(super) fn string(&mut self, key: &str) -> Result<Option<String>, String> {
        self.alias_string(key, &[])
    }

    pub(super) fn alias_string(
        &mut self,
        canonical: &str,
        aliases: &[&str],
    ) -> Result<Option<String>, String> {
        self.alias(canonical, aliases, |key, value| match value {
            Value::String(value) => Ok(value),
            Value::Number(value) => Ok(value.to_string()),
            Value::Bool(value) => Ok(value.to_string()),
            Value::Array(_) => Err(format!("link parameter {key:?} must not be repeated")),
            _ => Err(format!("link parameter {key:?} must be a string")),
        })
    }

    pub(super) fn boolean(&mut self, key: &str) -> Result<Option<bool>, String> {
        self.alias_bool(key, &[])
    }

    pub(super) fn alias_bool(
        &mut self,
        canonical: &str,
        aliases: &[&str],
    ) -> Result<Option<bool>, String> {
        self.alias(canonical, aliases, |key, value| match value {
            Value::Bool(value) => Ok(value),
            Value::Number(value) if value.as_i64() == Some(1) => Ok(true),
            Value::Number(value) if value.as_i64() == Some(0) => Ok(false),
            Value::String(value) => match value.to_ascii_lowercase().as_str() {
                "1" | "true" => Ok(true),
                "0" | "false" => Ok(false),
                _ => Err(format!(
                    "link parameter {key:?} must be true, false, 1, or 0"
                )),
            },
            Value::Array(_) => Err(format!("link parameter {key:?} must not be repeated")),
            _ => Err(format!("link parameter {key:?} must be a boolean")),
        })
    }

    pub(super) fn u64(&mut self, key: &str) -> Result<Option<u64>, String> {
        self.alias_u64(key, &[])
    }

    pub(super) fn i64(&mut self, key: &str) -> Result<Option<i64>, String> {
        self.alias(key, &[], |key, value| match value {
            Value::Number(value) => value
                .as_i64()
                .ok_or_else(|| format!("link parameter {key:?} must be an integer")),
            Value::String(value) => value
                .parse::<i64>()
                .map_err(|_| format!("link parameter {key:?} must be an integer")),
            Value::Array(_) => Err(format!("link parameter {key:?} must not be repeated")),
            _ => Err(format!("link parameter {key:?} must be an integer")),
        })
    }

    pub(super) fn alias_u64(
        &mut self,
        canonical: &str,
        aliases: &[&str],
    ) -> Result<Option<u64>, String> {
        self.alias(canonical, aliases, |key, value| match value {
            Value::Number(value) => value
                .as_u64()
                .ok_or_else(|| format!("link parameter {key:?} must be a non-negative integer")),
            Value::String(value) => value
                .parse::<u64>()
                .map_err(|_| format!("link parameter {key:?} must be a non-negative integer")),
            Value::Array(_) => Err(format!("link parameter {key:?} must not be repeated")),
            _ => Err(format!(
                "link parameter {key:?} must be a non-negative integer"
            )),
        })
    }

    pub(super) fn object(&mut self, key: &str) -> Result<Option<Map<String, Value>>, String> {
        let Some(value) = self.value(key) else {
            return Ok(None);
        };
        parse_object(key, value).map(Some)
    }

    pub(super) fn finish(self) -> Result<(), String> {
        let Some(key) = self.values.keys().next() else {
            return Ok(());
        };
        let guidance = if matches!(self.network.as_str(), "xhttp" | "splithttp") {
            "; future XHTTP fields must be encoded in the JSON `extra` parameter"
        } else {
            ""
        };
        Err(format!(
            "unsupported link parameter {key:?} for transport {:?}{guidance}; refusing to generate a potentially incomplete runtime config",
            self.network
        ))
    }

    fn alias<T: PartialEq>(
        &mut self,
        canonical: &str,
        aliases: &[&str],
        parse: impl Fn(&str, Value) -> Result<T, String>,
    ) -> Result<Option<T>, String> {
        let mut resolved: Option<(String, T)> = None;
        for key in aliases.iter().copied().chain(std::iter::once(canonical)) {
            let Some(value) = self.value(key) else {
                continue;
            };
            let parsed = parse(key, value)?;
            if let Some((previous_key, previous)) = &resolved
                && previous != &parsed
            {
                return Err(format!(
                    "conflicting link parameters {previous_key:?} and {key:?}"
                ));
            }
            resolved = Some((key.to_string(), parsed));
        }
        Ok(resolved.map(|(_, value)| value))
    }
}

fn parse_object(key: &str, value: Value) -> Result<Map<String, Value>, String> {
    let value = match value {
        Value::String(value) => serde_json::from_str(&value)
            .map_err(|error| format!("link parameter {key:?} must contain valid JSON: {error}"))?,
        Value::Array(_) => return Err(format!("link parameter {key:?} must not be repeated")),
        value => value,
    };
    value
        .as_object()
        .cloned()
        .ok_or_else(|| format!("link parameter {key:?} must be a JSON object"))
}
