use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

use super::{
    ApiObject, BurstObservatoryObject, DnsObject, EnvObject, FakeDnsObject, GeodataObject,
    LogObject, MetricsObject, ObservatoryObject, PolicyObject, ReverseObject, RoutingObject,
    StatsObject, VersionObject,
};
use crate::xray::parsing::protocols::{InboundObject, OutboundObject};
use crate::xray::parsing::transports::TransportObject;

#[derive(Debug, Error)]
pub enum XrayConfigError {
    #[error("invalid Xray JSON")]
    Json(#[from] serde_json::Error),
    #[error("unknown Xray fields: {0}")]
    UnknownFields(String),
    #[error("unsupported Xray schema: {0}")]
    UnsupportedSchema(String),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayConfig {
    pub env: Option<EnvObject>,
    pub version: Option<VersionObject>,
    pub log: Option<LogObject>,
    pub api: Option<ApiObject>,
    pub dns: Option<DnsObject>,
    pub routing: Option<RoutingObject>,
    pub policy: Option<PolicyObject>,
    pub inbounds: Option<Vec<InboundObject>>,
    pub outbounds: Option<Vec<OutboundObject>>,
    pub transport: Option<TransportObject>,
    pub stats: Option<StatsObject>,
    pub reverse: Option<ReverseObject>,
    #[serde(rename = "fakeDns")]
    pub fakedns: Option<FakeDnsObject>,
    pub metrics: Option<MetricsObject>,
    pub observatory: Option<ObservatoryObject>,
    pub burst_observatory: Option<BurstObservatoryObject>,
    pub geodata: Option<GeodataObject>,
    #[serde(skip)]
    pub(crate) source: Option<Value>,
}

impl Serialize for XrayConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut value = self
            .source
            .clone()
            .unwrap_or_else(|| Value::Object(Map::new()));
        merge_typed(
            &mut value,
            self.known_value().map_err(serde::ser::Error::custom)?,
        );
        value.serialize(serializer)
    }
}

impl XrayConfig {
    pub fn from_json_strict(json: &str) -> Result<Self, XrayConfigError> {
        let source: Value = serde_json::from_str(json)?;
        let mut parsed: Self = serde_json::from_value(source.clone())?;
        if let Some(inbound) = parsed.inbounds.as_ref().and_then(|inbounds| {
            inbounds
                .iter()
                .find(|inbound| inbound.has_unknown_protocol())
        }) {
            return Err(XrayConfigError::UnsupportedSchema(format!(
                "inbound protocol {}",
                inbound.protocol
            )));
        }
        let known = parsed.known_value()?;
        let mut unknown = Vec::new();
        collect_unknown_paths(&source, &known, "$", &mut unknown);
        if !unknown.is_empty() {
            return Err(XrayConfigError::UnknownFields(unknown.join(", ")));
        }
        parsed.source = None;
        Ok(parsed)
    }

    pub fn from_json_loose(json: &str) -> Result<Self, XrayConfigError> {
        let source: Value = serde_json::from_str(json)?;
        let mut parsed: Self = serde_json::from_value(source.clone())?;
        parsed.source = Some(source);
        Ok(parsed)
    }

    pub fn to_json(&self) -> Result<String, XrayConfigError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json_with_mode(
        json: &str,
        mode: crate::xray::parsing::ParseMode,
    ) -> Result<Self, XrayConfigError> {
        match mode {
            crate::xray::parsing::ParseMode::Strict => Self::from_json_strict(json),
            crate::xray::parsing::ParseMode::Lenient
            | crate::xray::parsing::ParseMode::Auto
            | crate::xray::parsing::ParseMode::Loose => Self::from_json_loose(json),
        }
    }

    fn known_value(&self) -> Result<Value, serde_json::Error> {
        let mut fields = Map::new();
        macro_rules! insert {
            ($field:ident, $name:literal) => {
                if let Some(value) = &self.$field {
                    fields.insert($name.to_string(), serde_json::to_value(value)?);
                }
            };
        }
        insert!(version, "version");
        insert!(env, "env");
        insert!(log, "log");
        insert!(api, "api");
        insert!(dns, "dns");
        insert!(routing, "routing");
        insert!(policy, "policy");
        insert!(inbounds, "inbounds");
        insert!(outbounds, "outbounds");
        insert!(transport, "transport");
        insert!(stats, "stats");
        insert!(reverse, "reverse");
        insert!(fakedns, "fakeDns");
        insert!(metrics, "metrics");
        insert!(observatory, "observatory");
        insert!(burst_observatory, "burstObservatory");
        insert!(geodata, "geodata");
        Ok(Value::Object(fields))
    }
}

fn merge_typed(target: &mut Value, typed: Value) {
    match (target, typed) {
        (Value::Object(target), Value::Object(typed)) => {
            for (key, value) in typed {
                match target.get_mut(&key) {
                    Some(existing) => merge_typed(existing, value),
                    None => match source_alias_for(&key).and_then(|alias| target.get_mut(alias)) {
                        Some(existing) => merge_typed(existing, value),
                        None => {
                            target.insert(key, value);
                        }
                    },
                }
            }
        }
        (Value::Array(target), Value::Array(typed)) => {
            let typed_len = typed.len();
            for (index, value) in typed.into_iter().enumerate() {
                if let Some(existing) = target.get_mut(index) {
                    merge_typed(existing, value);
                } else {
                    target.push(value);
                }
            }
            target.truncate(typed_len);
        }
        (target, typed) => *target = typed,
    }
}

fn collect_unknown_paths(source: &Value, known: &Value, path: &str, unknown: &mut Vec<String>) {
    match (source, known) {
        (Value::Object(source), Value::Object(known)) => {
            for (key, value) in source {
                if let Some(known_value) = known
                    .get(key)
                    .or_else(|| canonical_alias(key).and_then(|alias| known.get(alias)))
                {
                    collect_unknown_paths(value, known_value, &format!("{path}.{key}"), unknown);
                } else {
                    unknown.push(format!("{path}.{key}"));
                }
            }
        }
        (Value::Array(source), Value::Array(known)) => {
            for (index, value) in source.iter().enumerate() {
                if let Some(known_value) = known.get(index) {
                    collect_unknown_paths(value, known_value, &format!("{path}[{index}]"), unknown);
                }
            }
        }
        _ => {}
    }
}

fn canonical_alias(key: &str) -> Option<&'static str> {
    match key {
        "domains" => Some("domain"),
        "expectIPs" => Some("expectedIPs"),
        "source" => Some("sourceIP"),
        "tcpSettings" => Some("rawSettings"),
        "splithttpSettings" => Some("xhttpSettings"),
        "address" => Some("rewriteAddress"),
        "port" => Some("rewritePort"),
        "network" => Some("allowedNetwork"),
        "uuid" => Some("id"),
        _ => None,
    }
}

fn source_alias_for(key: &str) -> Option<&'static str> {
    match key {
        "domain" => Some("domains"),
        "expectedIPs" => Some("expectIPs"),
        "sourceIP" => Some("source"),
        "rawSettings" => Some("tcpSettings"),
        "xhttpSettings" => Some("splithttpSettings"),
        "rewriteAddress" => Some("address"),
        "rewritePort" => Some("port"),
        "allowedNetwork" => Some("network"),
        "id" => Some("uuid"),
        _ => None,
    }
}
