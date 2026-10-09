use crate::app::config::{DnsResolverPath, DnsSettings};
use crate::app::{AppError, Result};
use serde_json::{Value, json};
use xrat_engines::xray::config::{Inbound, Outbound, RoutingConfig, RoutingRule, XrayConfig};

use super::dns_validation;

pub(crate) fn xray_servers(dns: &DnsSettings) -> Result<Vec<Value>> {
    if dns.resolvers.is_empty() {
        return Ok(dns.servers.iter().map(|server| json!(server)).collect());
    }
    reject_overlapping_xray_rules(dns)?;
    let mut servers = Vec::new();
    for rule in &dns.rules {
        let resolver = dns
            .resolvers
            .iter()
            .find(|resolver| resolver.tag == rule.resolver)
            .ok_or_else(|| {
                AppError::InvalidArgument("[dns.rules].resolver does not exist".into())
            })?;
        let domains: Vec<String> = rule
            .domain
            .iter()
            .map(|domain| format!("full:{domain}"))
            .chain(
                rule.domain_suffix
                    .iter()
                    .map(|domain| format!("domain:{domain}")),
            )
            .collect();
        servers.push(xray_server(resolver, domains, true)?);
    }
    let final_resolver = dns
        .resolvers
        .iter()
        .find(|resolver| resolver.tag == dns.final_resolver)
        .ok_or_else(|| AppError::InvalidArgument("[dns].final_resolver does not exist".into()))?;
    servers.push(xray_server(final_resolver, Vec::new(), false)?);
    Ok(servers)
}

fn xray_server(
    resolver: &crate::app::config::DnsResolverSettings,
    domains: Vec<String>,
    skip: bool,
) -> Result<Value> {
    let url = dns_validation::endpoint(&resolver.address).map_err(AppError::InvalidArgument)?;
    let scheme = url.scheme().trim_end_matches("+local");
    if scheme == "tls" {
        return Err(AppError::InvalidArgument("[dns.resolvers].address DNS-over-TLS is not supported by the pinned Xray DNS schema; use HTTPS/TCP/UDP".into()));
    }
    let address = if scheme == "udp" {
        url.host_str()
            .unwrap_or_default()
            .trim_matches(['[', ']'])
            .to_string()
    } else {
        resolver.address.replace("+local://", "://")
    };
    let mut value =
        json!({"address": address, "domains": domains, "skipFallback": skip, "tag": resolver.tag});
    if scheme == "udp" {
        value["port"] = json!(url.port().unwrap_or(53));
    }
    Ok(value)
}

fn reject_overlapping_xray_rules(dns: &DnsSettings) -> Result<()> {
    for (index, rule) in dns.rules.iter().enumerate() {
        for earlier in &dns.rules[..index] {
            let overlap = rule.domain.iter().any(|domain| {
                earlier
                    .domain
                    .iter()
                    .any(|other| other.eq_ignore_ascii_case(domain))
                    || earlier
                        .domain_suffix
                        .iter()
                        .any(|suffix| suffix_matches(domain, suffix))
            }) || earlier.domain.iter().any(|domain| {
                rule.domain_suffix
                    .iter()
                    .any(|suffix| suffix_matches(domain, suffix))
            }) || rule.domain_suffix.iter().any(|suffix| {
                earlier
                    .domain_suffix
                    .iter()
                    .any(|other| suffix_matches(suffix, other) || suffix_matches(other, suffix))
            });
            if overlap {
                return Err(AppError::InvalidArgument("[dns.rules] overlapping policies cannot preserve first-match order in Xray DNS; use disjoint matches or sing-box".into()));
            }
        }
    }
    Ok(())
}

fn suffix_matches(domain: &str, suffix: &str) -> bool {
    domain.eq_ignore_ascii_case(suffix)
        || domain
            .to_ascii_lowercase()
            .ends_with(&format!(".{}", suffix.to_ascii_lowercase()))
}

pub(crate) fn apply_xray_runtime(
    config: &mut XrayConfig,
    dns: &DnsSettings,
    tun: bool,
) -> Result<()> {
    dns_validation::validate(dns, "xray", tun).map_err(AppError::InvalidArgument)?;
    if !dns.resolvers.is_empty() {
        ensure_direct(config);
        let rules = resolver_routes(dns);
        let existing = routing(config);
        for rule in rules.into_iter().rev() {
            if !existing.rules.iter().any(|item| {
                item.inbound_tag == rule.inbound_tag && item.outbound_tag == rule.outbound_tag
            }) {
                existing.rules.insert(0, rule);
            }
        }
    }
    if !dns.listener.enabled && dns.outbound.is_none() && !dns.fakeip.enabled {
        return Ok(());
    }
    let tag = dns
        .outbound
        .as_ref()
        .map(|outbound| outbound.tag.as_str())
        .unwrap_or("dns-out");
    let settings = if let Some(outbound) = &dns.outbound {
        let mut value = json!({"userLevel": outbound.user_level, "rules": outbound.rules.iter().map(|rule|
            json!({"action": rule.action, "domain": rule.domain, "qType": rule.query_type.iter()
                .map(u16::to_string).collect::<Vec<_>>().join(","), "rCode": rule.response_code})).collect::<Vec<_>>()});
        if let Some(network) = outbound.rewrite_network {
            value["rewriteNetwork"] = json!(network);
        }
        if let Some(address) = &outbound.rewrite_address {
            value["rewriteAddress"] = json!(address);
        }
        if let Some(port) = outbound.rewrite_port {
            value["rewritePort"] = json!(port);
        }
        // Empty qType means all types; omit rather than emit an empty port expression.
        if let Some(rules) = value["rules"].as_array_mut() {
            for rule in rules {
                if rule["qType"] == ""
                    && let Some(object) = rule.as_object_mut()
                {
                    object.remove("qType");
                }
            }
        }
        value
    } else {
        json!({})
    };
    if let Some(existing) = config
        .outbounds
        .iter_mut()
        .find(|outbound| outbound.tag == "dns-out" && outbound.protocol == "dns")
    {
        existing.tag = tag.into();
        existing.settings = settings;
        if let Some(routing) = &mut config.routing {
            for rule in &mut routing.rules {
                if rule.outbound_tag == "dns-out" {
                    rule.outbound_tag = tag.into();
                }
            }
        }
    } else {
        if config.outbounds.iter().any(|outbound| outbound.tag == tag) {
            return Err(AppError::InvalidArgument(
                "[dns.outbound].tag collides with an existing outbound".into(),
            ));
        }
        config.outbounds.push(Outbound {
            tag: tag.into(),
            protocol: "dns".into(),
            settings,
            stream_settings: None,
            mux: None,
        });
    }
    if dns.listener.enabled {
        if config
            .inbounds
            .iter()
            .any(|inbound| inbound.port == Some(dns.listener.port))
        {
            return Err(AppError::InvalidArgument(
                "[dns.listener].port collides with a managed inbound".into(),
            ));
        }
        config.inbounds.push(Inbound {
            sniffing: None,
            tag: "xrat-dns-in".into(),
            listen: Some(dns.listener.host.clone()),
            port: Some(dns.listener.port),
            protocol: "dokodemo-door".into(),
            settings: Some(json!({"address":"1.1.1.1", "port":53, "network":"tcp,udp"})),
        });
        routing(config)
            .rules
            .insert(0, rule(Some(vec!["xrat-dns-in".into()]), None, None, tag));
    }
    if dns.outbound.is_some() {
        let inbound_tags: Vec<_> = config
            .inbounds
            .iter()
            .filter(|inbound| inbound.tag != "api")
            .map(|inbound| inbound.tag.clone())
            .collect();
        routing(config).rules.insert(
            0,
            rule(
                Some(inbound_tags),
                Some("53".into()),
                Some("tcp,udp".into()),
                tag,
            ),
        );
    }
    if dns.fakeip.enabled {
        let mut pools =
            vec![json!({"ipPool":dns.fakeip.ipv4_range, "poolSize":dns.fakeip.pool_size})];
        if !dns.fakeip.ipv6_range.is_empty() {
            pools.push(json!({"ipPool":dns.fakeip.ipv6_range,"poolSize":dns.fakeip.pool_size}));
        }
        config.fake_dns = Some(pools);
        let dns_config = config.dns.as_mut().ok_or_else(|| {
            AppError::InvalidArgument("[dns.fakeip] requires DNS configuration".into())
        })?;
        dns_config.servers.insert(0, json!("fakedns"));
        dns_config.disable_fallback_if_match = Some(true);
        for inbound in &mut config.inbounds {
            if !matches!(inbound.tag.as_str(), "xrat-dns-in" | "api") {
                inbound.sniffing =
                    Some(json!({"enabled":true, "destOverride":["fakedns"], "routeOnly":false}));
            }
        }
    }
    Ok(())
}

fn ensure_direct(config: &mut XrayConfig) {
    if !config
        .outbounds
        .iter()
        .any(|outbound| outbound.tag == "direct")
    {
        config.outbounds.push(Outbound {
            tag: "direct".into(),
            protocol: "freedom".into(),
            settings: json!({}),
            stream_settings: None,
            mux: None,
        });
    }
}

pub(crate) fn resolver_routes(dns: &DnsSettings) -> Vec<RoutingRule> {
    dns.resolvers
        .iter()
        .map(|resolver| {
            rule(
                Some(vec![resolver.tag.clone()]),
                None,
                None,
                if resolver.path == DnsResolverPath::Proxy {
                    "proxy"
                } else {
                    "direct"
                },
            )
        })
        .collect()
}

fn routing(config: &mut XrayConfig) -> &mut RoutingConfig {
    config.routing.get_or_insert_with(|| RoutingConfig {
        domain_strategy: Some("AsIs".into()),
        rules: Vec::new(),
    })
}

fn rule(
    inbound_tag: Option<Vec<String>>,
    port: Option<String>,
    network: Option<String>,
    tag: &str,
) -> RoutingRule {
    RoutingRule {
        kind: "field".into(),
        domain: None,
        ip: None,
        port,
        network,
        inbound_tag,
        process: None,
        outbound_tag: tag.into(),
    }
}
