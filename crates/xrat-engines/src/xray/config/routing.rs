use serde_json::json;

use super::types::{Outbound, RoutingConfig, RoutingRule, XrayConfig};

#[derive(Debug, Clone)]
pub struct XrayRoutingOptions {
    pub domain_strategy: String,
    pub direct: XrayRouteList,
    pub block: XrayRouteList,
}

#[derive(Debug, Clone, Default)]
pub struct XrayRouteList {
    pub domain: Vec<String>,
    pub ip: Vec<String>,
    pub geosite: Vec<String>,
    pub geoip: Vec<String>,
}

impl XrayRouteList {
    fn domain_rules(&self) -> Vec<String> {
        self.domain
            .iter()
            .cloned()
            .chain(self.geosite.iter().map(|value| prefixed(value, "geosite:")))
            .collect()
    }

    fn ip_rules(&self) -> Vec<String> {
        self.ip
            .iter()
            .cloned()
            .chain(self.geoip.iter().map(|value| prefixed(value, "geoip:")))
            .collect()
    }

    fn is_empty(&self) -> bool {
        self.domain.is_empty()
            && self.ip.is_empty()
            && self.geosite.is_empty()
            && self.geoip.is_empty()
    }
}

pub(super) fn apply_runtime_routing(config: &mut XrayConfig, routing: Option<&XrayRoutingOptions>) {
    let Some(routing) =
        routing.filter(|routing| !routing.direct.is_empty() || !routing.block.is_empty())
    else {
        return;
    };

    let mut rules = Vec::new();
    append_route_rules(&mut rules, &routing.direct, "direct");
    append_route_rules(&mut rules, &routing.block, "block");

    if !routing.direct.is_empty() {
        config.outbounds.push(Outbound {
            tag: "direct".to_string(),
            protocol: "freedom".to_string(),
            settings: json!({}),
            stream_settings: None,
            mux: None,
        });
    }
    if !routing.block.is_empty() {
        config.outbounds.push(Outbound {
            tag: "block".to_string(),
            protocol: "blackhole".to_string(),
            settings: json!({}),
            stream_settings: None,
            mux: None,
        });
    }

    config.routing = Some(RoutingConfig {
        domain_strategy: Some(routing.domain_strategy.clone()),
        rules,
    });
}

fn append_route_rules(rules: &mut Vec<RoutingRule>, routes: &XrayRouteList, outbound_tag: &str) {
    let domains = routes.domain_rules();
    if !domains.is_empty() {
        rules.push(field_rule(Some(domains), None, None, outbound_tag));
    }

    let ips = routes.ip_rules();
    if !ips.is_empty() {
        rules.push(field_rule(None, Some(ips), None, outbound_tag));
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum XrayTunSplitMode {
    #[default]
    All,
    Blacklist,
    Whitelist,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct XrayTunSplitOptions {
    pub mode: XrayTunSplitMode,
    pub processes: Vec<String>,
}

pub fn enable_tun_split_routing(config: &mut XrayConfig, split: &XrayTunSplitOptions) {
    if split.mode == XrayTunSplitMode::All {
        return;
    }
    if split.mode == XrayTunSplitMode::Blacklist && split.processes.is_empty() {
        return;
    }

    if !config
        .outbounds
        .iter()
        .any(|outbound| outbound.tag == "direct")
    {
        config.outbounds.push(Outbound {
            tag: "direct".to_string(),
            protocol: "freedom".to_string(),
            settings: json!({}),
            stream_settings: None,
            mux: None,
        });
    }

    let routing = config.routing.get_or_insert_with(|| RoutingConfig {
        domain_strategy: None,
        rules: Vec::new(),
    });

    let insert_idx = routing
        .rules
        .iter()
        .position(|rule| rule.outbound_tag == "api")
        .map_or(0, |idx| idx + 1);

    let mut split_rules = Vec::new();
    match split.mode {
        XrayTunSplitMode::All => {}
        XrayTunSplitMode::Blacklist => {
            split_rules.push(RoutingRule {
                kind: "field".to_string(),
                domain: None,
                ip: None,
                port: None,
                network: None,
                inbound_tag: Some(vec!["tun-in".to_string()]),
                process: Some(split.processes.clone()),
                outbound_tag: "direct".to_string(),
            });
        }
        XrayTunSplitMode::Whitelist => {
            if !routing.rules.iter().any(|rule| {
                rule.outbound_tag == "direct"
                    && rule
                        .process
                        .as_ref()
                        .is_some_and(|procs| procs.iter().any(|proc| proc == "self/"))
            }) {
                split_rules.push(RoutingRule {
                    kind: "field".to_string(),
                    domain: None,
                    ip: None,
                    port: None,
                    network: None,
                    inbound_tag: Some(vec!["tun-in".to_string()]),
                    process: Some(vec!["self/".to_string()]),
                    outbound_tag: "direct".to_string(),
                });
            }
            if !split.processes.is_empty() {
                split_rules.push(RoutingRule {
                    kind: "field".to_string(),
                    domain: None,
                    ip: None,
                    port: None,
                    network: None,
                    inbound_tag: Some(vec!["tun-in".to_string()]),
                    process: Some(split.processes.clone()),
                    outbound_tag: "proxy".to_string(),
                });
            }
            split_rules.push(RoutingRule {
                kind: "field".to_string(),
                domain: None,
                ip: None,
                port: None,
                network: None,
                inbound_tag: Some(vec!["tun-in".to_string()]),
                process: None,
                outbound_tag: "direct".to_string(),
            });
        }
    }
    if !split_rules.is_empty() {
        routing.rules.splice(insert_idx..insert_idx, split_rules);
    }
}

pub(super) fn field_rule(
    domain: Option<Vec<String>>,
    ip: Option<Vec<String>>,
    inbound_tag: Option<Vec<String>>,
    outbound_tag: &str,
) -> RoutingRule {
    RoutingRule {
        kind: "field".to_string(),
        domain,
        ip,
        port: None,
        network: None,
        inbound_tag,
        process: None,
        outbound_tag: outbound_tag.to_string(),
    }
}

fn prefixed(value: &str, prefix: &str) -> String {
    if value.starts_with(prefix) {
        value.to_string()
    } else {
        format!("{prefix}{value}")
    }
}
