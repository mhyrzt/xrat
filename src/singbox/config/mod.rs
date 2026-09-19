use serde::{Deserialize, Serialize};

use crate::model::{Node, Protocol};

mod hy2;
mod simple;
mod transport;
mod trojan;
mod vless;
mod vmess;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxConfig {
    pub log: SingboxLogConfig,
    pub inbounds: Vec<SingboxInbound>,
    pub outbounds: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns: Option<SingboxDnsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<SingboxRoute>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<SingboxExperimental>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxDnsConfig {
    pub servers: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<serde_json::Value>,
    #[serde(rename = "final")]
    pub final_server: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_cache: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxRoute {
    pub rules: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rule_set: Vec<serde_json::Value>,
    #[serde(rename = "final")]
    pub final_outbound: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_domain_resolver: Option<String>,
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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SingboxInbound {
    Socks {
        tag: String,
        listen: String,
        listen_port: u16,
        #[serde(skip_serializing_if = "Option::is_none")]
        users: Option<Vec<SingboxInboundUser>>,
    },
    Http {
        tag: String,
        listen: String,
        listen_port: u16,
    },
    Shadowsocks {
        tag: String,
        listen: String,
        listen_port: u16,
        network: String,
        method: String,
        password: String,
    },
}

impl SingboxInbound {
    pub fn socks(
        tag: impl Into<String>,
        listen: impl Into<String>,
        listen_port: u16,
        users: Option<Vec<SingboxInboundUser>>,
    ) -> Self {
        Self::Socks {
            tag: tag.into(),
            listen: listen.into(),
            listen_port,
            users,
        }
    }

    pub fn http(tag: impl Into<String>, listen: impl Into<String>, listen_port: u16) -> Self {
        Self::Http {
            tag: tag.into(),
            listen: listen.into(),
            listen_port,
        }
    }

    pub fn shadowsocks(
        tag: impl Into<String>,
        listen: impl Into<String>,
        listen_port: u16,
        network: impl Into<String>,
        method: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Self, String> {
        let network = network.into();
        if !matches!(network.as_str(), "tcp" | "udp") {
            return Err(format!(
                "sing-box Shadowsocks inbound network must be tcp or udp; got {network:?}"
            ));
        }
        let method = method.into();
        if !SHADOWSOCKS_METHODS.contains(&method.as_str()) {
            return Err(format!(
                "unsupported Shadowsocks inbound method {method:?} for sing-box 1.13"
            ));
        }
        let password = password.into();
        if password.is_empty() {
            return Err("Shadowsocks inbound requires a password".to_string());
        }
        Ok(Self::Shadowsocks {
            tag: tag.into(),
            listen: listen.into(),
            listen_port,
            network,
            method,
            password,
        })
    }
}

const SHADOWSOCKS_METHODS: &[&str] = &[
    "2022-blake3-aes-128-gcm",
    "2022-blake3-aes-256-gcm",
    "2022-blake3-chacha20-poly1305",
    "none",
    "aes-128-gcm",
    "aes-192-gcm",
    "aes-256-gcm",
    "chacha20-ietf-poly1305",
    "xchacha20-ietf-poly1305",
    "aes-128-ctr",
    "aes-192-ctr",
    "aes-256-ctr",
    "aes-128-cfb",
    "aes-192-cfb",
    "aes-256-cfb",
    "rc4-md5",
    "chacha20-ietf",
    "xchacha20",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxInboundUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxExperimental {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clash_api: Option<SingboxClashApi>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_file: Option<SingboxCacheFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxCacheFile {
    pub enabled: bool,
    pub path: String,
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
        Protocol::Vless => vless::build_vless_outbound(node)?,
        Protocol::Vmess => vmess::build_vmess_outbound(node)?,
        Protocol::Trojan => trojan::build_trojan_outbound(node)?,
        Protocol::Ss | Protocol::Http | Protocol::Socks5 => simple::build_outbound(node)?,
    };

    Ok(SingboxConfig {
        log: SingboxLogConfig {
            level: "warn".to_string(),
            timestamp: true,
        },
        inbounds: vec![SingboxInbound::socks(
            "socks-in",
            "127.0.0.1",
            local_port,
            None,
        )],
        outbounds: vec![
            outbound,
            serde_json::json!({"type": "direct", "tag": "direct"}),
        ],
        dns: None,
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
    generate_singbox_runtime_config_with_dns(node, inbounds, clash_api, routing, None)
}

pub fn generate_singbox_runtime_config_with_dns(
    node: &Node,
    inbounds: Vec<SingboxInbound>,
    clash_api: Option<SingboxClashApi>,
    routing: Option<&SingboxRoutingOptions>,
    dns: Option<&SingboxDnsConfig>,
) -> Result<SingboxConfig, String> {
    let outbound = match node.protocol {
        Protocol::Hy2 => hy2::build_hy2_outbound(node)?,
        Protocol::Vless => vless::build_vless_outbound(node)?,
        Protocol::Vmess => vmess::build_vmess_outbound(node)?,
        Protocol::Trojan => trojan::build_trojan_outbound(node)?,
        Protocol::Ss | Protocol::Http | Protocol::Socks5 => simple::build_outbound(node)?,
    };

    let mut outbounds = vec![
        outbound,
        serde_json::json!({"type": "direct", "tag": "direct"}),
    ];
    let mut route = build_route(routing)?;
    if route
        .as_ref()
        .is_some_and(|route| route.rules.iter().any(|rule| rule["outbound"] == "block"))
    {
        outbounds.push(serde_json::json!({"type": "block", "tag": "block"}));
    }
    // sing-box 1.13 requires a route-level default domain resolver whenever a
    // DNS server or outbound dials by name; a TLS DNS server triggers it even
    // for IP endpoints. Point it at a local resolver when present, otherwise
    // the first configured server.
    if let Some(dns) = dns {
        let resolver = dns
            .servers
            .iter()
            .find(|server| server["type"] == "local")
            .or_else(|| dns.servers.first())
            .and_then(|server| server["tag"].as_str())
            .map(str::to_string);
        let route = route.get_or_insert(SingboxRoute {
            rules: Vec::new(),
            rule_set: Vec::new(),
            final_outbound: "proxy".to_string(),
            default_domain_resolver: None,
        });
        route.default_domain_resolver = resolver;
    }

    Ok(SingboxConfig {
        log: SingboxLogConfig {
            level: "warn".to_string(),
            timestamp: true,
        },
        inbounds,
        outbounds,
        dns: dns.cloned(),
        route,
        experimental: clash_api.map(|clash_api| SingboxExperimental {
            clash_api: Some(clash_api),
            cache_file: None,
        }),
    })
}

impl SingboxConfig {
    /// Whether the generated route declares any rule-set (currently remote
    /// SagerNet rule-sets), which requires a cache file to persist downloads.
    pub fn has_rule_sets(&self) -> bool {
        self.route
            .as_ref()
            .is_some_and(|route| !route.rule_set.is_empty())
    }

    /// Enable the experimental cache file used to persist remote rule-sets.
    pub fn enable_cache_file(&mut self, path: String) {
        let experimental = self.experimental.get_or_insert(SingboxExperimental {
            clash_api: None,
            cache_file: None,
        });
        experimental.cache_file = Some(SingboxCacheFile {
            enabled: true,
            path,
        });
    }
}

fn build_route(routing: Option<&SingboxRoutingOptions>) -> Result<Option<SingboxRoute>, String> {
    let Some(routing) = routing else {
        return Ok(None);
    };

    let mut rules = Vec::new();
    let mut rule_sets = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    append_route_rules(
        &mut rules,
        &mut rule_sets,
        &mut seen,
        &routing.direct,
        "direct",
        "routing.direct",
    )?;
    append_route_rules(
        &mut rules,
        &mut rule_sets,
        &mut seen,
        &routing.block,
        "block",
        "routing.block",
    )?;
    if rules.is_empty() {
        return Ok(None);
    }

    Ok(Some(SingboxRoute {
        rules,
        rule_set: rule_sets,
        final_outbound: "proxy".to_string(),
        default_domain_resolver: None,
    }))
}

fn append_route_rules(
    rules: &mut Vec<serde_json::Value>,
    rule_sets: &mut Vec<serde_json::Value>,
    seen: &mut std::collections::BTreeSet<String>,
    routes: &SingboxRouteList,
    outbound: &str,
    field: &str,
) -> Result<(), String> {
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

    let tags = append_rule_set_rules(rule_sets, seen, routes, field)?;
    if !tags.is_empty() {
        rules.push(serde_json::json!({
            "rule_set": tags,
            "action": "route",
            "outbound": outbound,
        }));
    }

    Ok(())
}

const GEOSITE_RULE_SET_BASE: &str =
    "https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set";
const GEOIP_RULE_SET_BASE: &str = "https://raw.githubusercontent.com/SagerNet/sing-geoip/rule-set";

/// Register remote SagerNet rule-sets for the configured geosite/geoip
/// categories and return their tags for the referencing route rule.
fn append_rule_set_rules(
    rule_sets: &mut Vec<serde_json::Value>,
    seen: &mut std::collections::BTreeSet<String>,
    routes: &SingboxRouteList,
    field: &str,
) -> Result<Vec<String>, String> {
    let mut tags = Vec::new();
    for (kind, base, names) in [
        ("geosite", GEOSITE_RULE_SET_BASE, &routes.geosite),
        ("geoip", GEOIP_RULE_SET_BASE, &routes.geoip),
    ] {
        for name in names {
            validate_rule_set_name(kind, name, field)?;
            let tag = format!("{kind}-{name}");
            if seen.insert(tag.clone()) {
                rule_sets.push(serde_json::json!({
                    "type": "remote",
                    "tag": tag,
                    "format": "binary",
                    "url": format!("{base}/{kind}-{name}.srs"),
                }));
            }
            tags.push(tag);
        }
    }
    Ok(tags)
}

fn validate_rule_set_name(kind: &str, name: &str, field: &str) -> Result<(), String> {
    let valid = !name.is_empty()
        && name.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        });
    if valid {
        return Ok(());
    }
    Err(format!(
        "{field}.{kind} entry {name:?} is not a valid rule-set category name; use letters, digits, '-', '_', or '.'"
    ))
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
