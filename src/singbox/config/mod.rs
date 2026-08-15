use serde::{Deserialize, Serialize};

use crate::model::{Node, Protocol};

mod hy2;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxConfig {
    pub log: SingboxLogConfig,
    pub inbounds: Vec<SingboxInbound>,
    pub outbounds: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<SingboxRoute>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<SingboxExperimental>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxRoute {
    pub rules: Vec<serde_json::Value>,
    #[serde(rename = "final")]
    pub final_outbound: String,
}

#[derive(Debug, Clone, Default)]
pub struct SingboxRoutingOptions {
    pub direct: SingboxRouteList,
    pub block: SingboxRouteList,
}

#[derive(Debug, Clone, Default)]
pub struct SingboxRouteList {
    pub domain: Vec<String>,
    pub ip: Vec<String>,
    pub geosite: Vec<String>,
    pub geoip: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxLogConfig {
    pub level: String,
    /// sing-box omits timestamps by default. Enable them in generated configs so
    /// the TUI engine tab can parse a real time column for sing-box log lines.
    pub timestamp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxInbound {
    #[serde(rename = "type")]
    pub kind: String,
    pub tag: String,
    pub listen: String,
    pub listen_port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<SingboxInboundUser>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxInboundUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxExperimental {
    pub clash_api: SingboxClashApi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxClashApi {
    pub external_controller: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
}

pub fn generate_singbox_probe_config(
    node: &Node,
    local_port: u16,
) -> Result<SingboxConfig, String> {
    let outbound = match node.protocol {
        Protocol::Hy2 => hy2::build_hy2_outbound(node)?,
        _ => {
            return Err(format!(
                "protocol {} is not implemented for sing-box output",
                node.protocol
            ));
        }
    };

    Ok(SingboxConfig {
        log: SingboxLogConfig {
            level: "warn".to_string(),
            timestamp: true,
        },
        inbounds: vec![SingboxInbound {
            kind: "socks".to_string(),
            tag: "socks-in".to_string(),
            listen: "127.0.0.1".to_string(),
            listen_port: local_port,
            network: None,
            method: None,
            password: None,
            users: None,
        }],
        outbounds: vec![
            outbound,
            serde_json::json!({"type": "direct", "tag": "direct"}),
        ],
        route: None,
        experimental: None,
    })
}

pub fn generate_singbox_runtime_config(
    node: &Node,
    inbounds: Vec<SingboxInbound>,
    clash_api: Option<SingboxClashApi>,
    routing: Option<&SingboxRoutingOptions>,
) -> Result<SingboxConfig, String> {
    let outbound = match node.protocol {
        Protocol::Hy2 => hy2::build_hy2_outbound(node)?,
        _ => {
            return Err(format!(
                "protocol {} is not implemented for sing-box runtime output",
                node.protocol
            ));
        }
    };

    let mut outbounds = vec![
        outbound,
        serde_json::json!({"type": "direct", "tag": "direct"}),
    ];
    let route = build_route(routing)?;
    if route
        .as_ref()
        .is_some_and(|route| route.rules.iter().any(|rule| rule["outbound"] == "block"))
    {
        outbounds.push(serde_json::json!({"type": "block", "tag": "block"}));
    }

    Ok(SingboxConfig {
        log: SingboxLogConfig {
            level: "warn".to_string(),
            timestamp: true,
        },
        inbounds,
        outbounds,
        route,
        experimental: clash_api.map(|clash_api| SingboxExperimental { clash_api }),
    })
}

fn build_route(routing: Option<&SingboxRoutingOptions>) -> Result<Option<SingboxRoute>, String> {
    let Some(routing) = routing else {
        return Ok(None);
    };

    let mut rules = Vec::new();
    append_route_rules(&mut rules, &routing.direct, "direct", "routing.direct")?;
    append_route_rules(&mut rules, &routing.block, "block", "routing.block")?;
    if rules.is_empty() {
        return Ok(None);
    }

    Ok(Some(SingboxRoute {
        rules,
        final_outbound: "proxy".to_string(),
    }))
}

fn append_route_rules(
    rules: &mut Vec<serde_json::Value>,
    routes: &SingboxRouteList,
    outbound: &str,
    field: &str,
) -> Result<(), String> {
    if !routes.geosite.is_empty() || !routes.geoip.is_empty() {
        return Err(format!(
            "{field}.geosite/geoip require sing-box rule-set support; use Xray/V2Ray or remove those entries"
        ));
    }

    if !routes.domain.is_empty() {
        let mut exact = Vec::new();
        let mut suffix = Vec::new();
        let mut keyword = Vec::new();
        let mut regex = Vec::new();
        for rule in &routes.domain {
            if let Some(value) = rule.strip_prefix("full:") {
                exact.push(value.to_string());
            } else if let Some(value) = rule.strip_prefix("domain:") {
                suffix.push(value.to_string());
            } else if let Some(value) = rule.strip_prefix("keyword:") {
                keyword.push(value.to_string());
            } else if let Some(value) = rule.strip_prefix("regexp:") {
                regex.push(value.to_string());
            } else if rule.starts_with("geosite:")
                || rule.starts_with("ext:")
                || rule.starts_with("dotless:")
            {
                return Err(format!(
                    "{field}.domain entry {rule:?} is not translatable to sing-box; use a supported domain rule or Xray/V2Ray"
                ));
            } else {
                keyword.push(rule.clone());
            }
        }

        let mut object = serde_json::Map::new();
        insert_non_empty(&mut object, "domain", exact);
        insert_non_empty(&mut object, "domain_suffix", suffix);
        insert_non_empty(&mut object, "domain_keyword", keyword);
        insert_non_empty(&mut object, "domain_regex", regex);
        object.insert("action".to_string(), serde_json::json!("route"));
        object.insert("outbound".to_string(), serde_json::json!(outbound));
        rules.push(serde_json::Value::Object(object));
    }

    if !routes.ip.is_empty() {
        for rule in &routes.ip {
            if rule.starts_with('!')
                || rule.starts_with("geoip:")
                || rule.starts_with("ext:")
                || rule.starts_with("ext-ip:")
            {
                return Err(format!(
                    "{field}.ip entry {rule:?} is not translatable to sing-box; use an IP/CIDR or Xray/V2Ray"
                ));
            }
        }
        rules.push(serde_json::json!({
            "ip_cidr": routes.ip,
            "action": "route",
            "outbound": outbound,
        }));
    }

    Ok(())
}

fn insert_non_empty(
    object: &mut serde_json::Map<String, serde_json::Value>,
    field: &str,
    values: Vec<String>,
) {
    if !values.is_empty() {
        object.insert(field.to_string(), serde_json::json!(values));
    }
}
