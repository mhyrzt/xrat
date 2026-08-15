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
        inbound_tag,
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
