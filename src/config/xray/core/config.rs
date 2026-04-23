use serde::{Deserialize, Serialize};

use super::{
    ApiObject, BurstObservatoryObject, DnsObject, FakeDnsObject, LogObject, MetricsObject,
    ObservatoryObject, PolicyObject, ReverseObject, RoutingObject, StatsObject, VersionObject,
};
use crate::config::xray::protocols::{InboundObject, OutboundObject};
use crate::config::xray::transports::TransportObject;

/// Main Xray configuration object
///
/// This struct supports both strict and loose parsing modes:
/// - Strict mode: Use `from_json_strict` which rejects unknown fields
/// - Loose mode: Use `from_json_loose` which allows unknown fields
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<VersionObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log: Option<LogObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<ApiObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns: Option<DnsObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<RoutingObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<PolicyObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbounds: Option<Vec<InboundObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outbounds: Option<Vec<OutboundObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<TransportObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<StatsObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverse: Option<ReverseObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fakedns: Option<FakeDnsObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<MetricsObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observatory: Option<ObservatoryObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub burst_observatory: Option<BurstObservatoryObject>,
}

/// Strict version of XrayConfig that denies unknown fields
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct XrayConfigStrict {
    version: Option<VersionObject>,
    log: Option<LogObject>,
    api: Option<ApiObject>,
    dns: Option<DnsObject>,
    routing: Option<RoutingObject>,
    policy: Option<PolicyObject>,
    inbounds: Option<Vec<InboundObject>>,
    outbounds: Option<Vec<OutboundObject>>,
    transport: Option<TransportObject>,
    stats: Option<StatsObject>,
    reverse: Option<ReverseObject>,
    fakedns: Option<FakeDnsObject>,
    metrics: Option<MetricsObject>,
    observatory: Option<ObservatoryObject>,
    burst_observatory: Option<BurstObservatoryObject>,
}

impl From<XrayConfigStrict> for XrayConfig {
    fn from(strict: XrayConfigStrict) -> Self {
        XrayConfig {
            version: strict.version,
            log: strict.log,
            api: strict.api,
            dns: strict.dns,
            routing: strict.routing,
            policy: strict.policy,
            inbounds: strict.inbounds,
            outbounds: strict.outbounds,
            transport: strict.transport,
            stats: strict.stats,
            reverse: strict.reverse,
            fakedns: strict.fakedns,
            metrics: strict.metrics,
            observatory: strict.observatory,
            burst_observatory: strict.burst_observatory,
        }
    }
}

impl XrayConfig {
    /// Parse from JSON string in strict mode (rejects unknown fields)
    pub fn from_json_strict(json: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let strict: XrayConfigStrict = serde_json::from_str(json)?;
        Ok(strict.into())
    }

    /// Parse from JSON string in loose mode (allows unknown fields)
    pub fn from_json_loose(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Parse from JSON with specified mode
    pub fn from_json_with_mode(
        json: &str,
        mode: crate::config::xray::ParseMode,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        match mode {
            crate::config::xray::ParseMode::Strict => Self::from_json_strict(json),
            crate::config::xray::ParseMode::Loose => {
                Self::from_json_loose(json).map_err(|e| e.into())
            }
        }
    }
}
