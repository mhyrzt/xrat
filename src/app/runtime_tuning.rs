//! Bridges runtime tuning and routing app-config sections to generated engine
//! options, plus inbound `listen_interface` resolution.

use std::net::IpAddr;

use crate::app::AppError;
use crate::app::config::{DnsHostValue, DnsSettings, RouteList, RoutingSettings, RuntimeSettings};
use crate::singbox::{SingboxDnsConfig, SingboxRouteList, SingboxRoutingOptions};
use crate::xray::{
    FragmentOptions, MuxOptions, XrayDnsConfig, XrayDnsHostValue, XrayGenOptions, XrayRouteList,
    XrayRoutingOptions,
};
use serde_json::{Value, json};
use url::Url;

const SINGBOX_LOCAL_DNS_TAG: &str = "xrat-dns-local";
const SINGBOX_HOSTS_DNS_TAG: &str = "xrat-dns-hosts";

/// Translate runtime tuning settings into outbound generation options. Routing
/// is added separately for managed sessions so probe configs remain proxy-only.
pub(crate) fn build_xray_gen_options(runtime: &RuntimeSettings) -> XrayGenOptions {
    let mux = runtime.mux.enabled.then(|| MuxOptions {
        concurrency: runtime.mux.concurrency,
        xudp_concurrency: runtime.mux.xudp_concurrency,
        xudp_proxy_udp443: runtime.mux.xudp_proxy_udp443.clone(),
    });
    let fragment = runtime.fragment.enabled.then(|| FragmentOptions {
        packets: fragment_packets(runtime),
        length: format_range(runtime.fragment.length),
        interval: format_range(runtime.fragment.interval),
    });

    XrayGenOptions {
        mux,
        fragment,
        interface: non_empty(&runtime.network.interface),
        mark: (runtime.network.mark != 0).then_some(runtime.network.mark),
        bind_address: non_empty(&runtime.network.bind_address),
        routing: None,
        dns: None,
    }
}

pub(crate) fn apply_xray_dns_options(
    options: &mut XrayGenOptions,
    dns: &DnsSettings,
) -> crate::app::Result<()> {
    if dns == &DnsSettings::default() {
        return Ok(());
    }

    let query_strategy = match dns.query_strategy.as_str() {
        "UseIP" | "UseIPv4" | "UseIPv6" | "UseSystem" => dns.query_strategy.clone(),
        other => {
            return Err(AppError::InvalidArgument(format!(
                "[dns].query_strategy must be UseIP, UseIPv4, UseIPv6, or UseSystem; got \"{other}\""
            )));
        }
    };

    let mut servers = Vec::with_capacity(dns.servers.len());
    for server in &dns.servers {
        let server = server.trim();
        if server.is_empty() {
            return Err(AppError::InvalidArgument(
                "[dns].servers cannot contain an empty server".to_string(),
            ));
        }
        servers.push(server.to_string());
    }

    let hosts = dns
        .hosts
        .iter()
        .map(|(host, value)| {
            let value = match value {
                DnsHostValue::One(value) => XrayDnsHostValue::One(value.clone()),
                DnsHostValue::Many(value) => XrayDnsHostValue::Many(value.clone()),
            };
            (host.clone(), value)
        })
        .collect();

    options.dns = Some(XrayDnsConfig {
        servers,
        hosts,
        query_strategy,
        use_system_hosts: dns.use_system_hosts,
        disable_cache: dns.disable_cache,
        disable_fallback: dns.disable_fallback,
        enable_parallel_query: dns.enable_parallel_query,
    });
    Ok(())
}

pub(crate) fn build_singbox_dns_options(
    dns: &DnsSettings,
) -> crate::app::Result<Option<SingboxDnsConfig>> {
    if dns == &DnsSettings::default() {
        return Ok(None);
    }

    let strategy = match dns.query_strategy.as_str() {
        "UseIPv4" => "ipv4_only",
        "UseIPv6" => "ipv6_only",
        "UseIP" | "UseSystem" => {
            return Err(AppError::InvalidArgument(format!(
                "[dns].query_strategy = \"{}\" has no exact modern sing-box equivalent; use UseIPv4 or UseIPv6 for sing-box sessions",
                dns.query_strategy
            )));
        }
        other => {
            return Err(AppError::InvalidArgument(format!(
                "[dns].query_strategy must be UseIPv4 or UseIPv6 for sing-box sessions; got \"{other}\""
            )));
        }
    };

    if dns.disable_fallback {
        return Err(AppError::InvalidArgument(
            "[dns].disable_fallback is Xray/V2Ray-only and cannot be represented safely in sing-box"
                .to_string(),
        ));
    }
    if !dns.enable_parallel_query {
        return Err(AppError::InvalidArgument(
            "[dns].enable_parallel_query = false is Xray/V2Ray-only and cannot be represented safely in sing-box"
                .to_string(),
        ));
    }

    let mut servers = Vec::new();
    let mut final_server = None;
    let mut needs_local_resolver = false;
    for (index, server) in dns.servers.iter().enumerate() {
        let tag = format!("xrat-dns-{index}");
        let (value, needs_resolver) = singbox_dns_server(server, &tag)?;
        if final_server.is_none() {
            final_server = Some(tag);
        }
        needs_local_resolver |= needs_resolver;
        servers.push(value);
    }

    if needs_local_resolver {
        servers.push(json!({
            "type": "local",
            "tag": SINGBOX_LOCAL_DNS_TAG,
        }));
    }

    if final_server.is_none() {
        final_server = Some(SINGBOX_LOCAL_DNS_TAG.to_string());
        servers.push(json!({
            "type": "local",
            "tag": SINGBOX_LOCAL_DNS_TAG,
        }));
    }

    let mut rules = Vec::new();
    let mut predefined = serde_json::Map::new();
    let mut exact_hosts = Vec::new();
    for (host, value) in &dns.hosts {
        let host = singbox_exact_host(host)?;
        exact_hosts.push(host.clone());
        predefined.insert(host, singbox_host_value(value)?);
    }

    if dns.use_system_hosts || !predefined.is_empty() {
        let mut hosts_server = json!({
            "type": "hosts",
            "tag": SINGBOX_HOSTS_DNS_TAG,
            "predefined": predefined,
        });
        if !dns.use_system_hosts {
            hosts_server["path"] = json!([]);
        }
        servers.push(hosts_server);

        if !exact_hosts.is_empty() {
            rules.push(json!({
                "domain": exact_hosts,
                "action": "route",
                "server": SINGBOX_HOSTS_DNS_TAG,
            }));
        }
        if dns.use_system_hosts {
            rules.push(json!({
                "ip_accept_any": true,
                "action": "route",
                "server": SINGBOX_HOSTS_DNS_TAG,
            }));
        }
    }

    Ok(Some(SingboxDnsConfig {
        servers,
        rules,
        final_server: final_server.expect("a local fallback is always added"),
        strategy: Some(strategy.to_string()),
        disable_cache: dns.disable_cache.then_some(true),
    }))
}

fn singbox_dns_server(raw: &str, tag: &str) -> crate::app::Result<(Value, bool)> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(AppError::InvalidArgument(
            "[dns].servers cannot contain an empty server".to_string(),
        ));
    }
    if raw == "localhost" {
        return Ok((json!({"type": "local", "tag": tag}), false));
    }
    if raw == "fakedns" {
        return Err(AppError::InvalidArgument(
            "[dns].servers entry \"fakedns\" is not supported by the generated sing-box DNS configuration"
                .to_string(),
        ));
    }

    let (scheme, rest) = raw.split_once("://").unwrap_or(("udp", raw));
    let scheme = scheme.to_ascii_lowercase();
    let kind = match scheme.as_str() {
        "udp" | "udp+local" => "udp",
        "tcp" | "tcp+local" => "tcp",
        "tls" | "tls+local" => "tls",
        "quic" | "quic+local" => "quic",
        "https" | "https+local" => "https",
        "h3" | "h3+local" => "h3",
        "http" | "http+local" | "h2c" | "h2c+local" => {
            return Err(AppError::InvalidArgument(format!(
                "[dns].servers entry \"{raw}\" uses {scheme}, which has no safe modern sing-box mapping"
            )));
        }
        _ => {
            return Err(AppError::InvalidArgument(format!(
                "[dns].servers entry \"{raw}\" uses unsupported scheme \"{scheme}\""
            )));
        }
    };

    let url = Url::parse(&format!("{kind}://{rest}")).map_err(|error| {
        AppError::InvalidArgument(format!(
            "[dns].servers entry \"{raw}\" is not a valid {kind} DNS endpoint: {error}"
        ))
    })?;
    if !url.username().is_empty() || url.password().is_some() {
        return Err(AppError::InvalidArgument(format!(
            "[dns].servers entry \"{raw}\" cannot contain credentials"
        )));
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(AppError::InvalidArgument(format!(
            "[dns].servers entry \"{raw}\" cannot contain a query or fragment"
        )));
    }

    let host = url.host_str().ok_or_else(|| {
        AppError::InvalidArgument(format!(
            "[dns].servers entry \"{raw}\" is missing a server host"
        ))
    })?;
    let is_https = matches!(kind, "https" | "h3");
    let path = url.path();
    if !is_https && !path.is_empty() && path != "/" {
        return Err(AppError::InvalidArgument(format!(
            "[dns].servers entry \"{raw}\" has a path, but {kind} endpoints do not support one"
        )));
    }

    let default_port = match kind {
        "tls" | "quic" => 853,
        "https" | "h3" => 443,
        _ => 53,
    };
    let port = url.port().unwrap_or(default_port);
    let needs_resolver = host.parse::<IpAddr>().is_err();
    let mut value = json!({
        "type": kind,
        "tag": tag,
        "server": host,
        "server_port": port,
    });
    if needs_resolver {
        value["domain_resolver"] = json!(SINGBOX_LOCAL_DNS_TAG);
    }
    if matches!(kind, "tls" | "quic" | "https" | "h3") {
        value["tls"] = json!({"server_name": host});
    }
    if is_https {
        value["path"] = json!(if path == "/" { "/dns-query" } else { path });
    }
    Ok((value, needs_resolver))
}

fn singbox_exact_host(host: &str) -> crate::app::Result<String> {
    if let Some(host) = host.strip_prefix("full:") {
        if host.is_empty() {
            return Err(AppError::InvalidArgument(
                "[dns.hosts] contains an empty full: hostname".to_string(),
            ));
        }
        return Ok(host.to_string());
    }
    if host.is_empty()
        || host.starts_with("domain:")
        || host.starts_with("keyword:")
        || host.starts_with("regexp:")
        || host.starts_with("geosite:")
        || host.starts_with("ext:")
        || host.starts_with("dotless:")
    {
        return Err(AppError::InvalidArgument(format!(
            "[dns.hosts] key \"{host}\" is not an exact hostname; sing-box supports only plain and full: keys"
        )));
    }
    Ok(host.to_string())
}

fn singbox_host_value(value: &DnsHostValue) -> crate::app::Result<Value> {
    let values = match value {
        DnsHostValue::One(value) => vec![value],
        DnsHostValue::Many(values) => values.iter().collect(),
    };
    if values.is_empty() || values.iter().any(|value| value.parse::<IpAddr>().is_err()) {
        return Err(AppError::InvalidArgument(
            "[dns.hosts] sing-box values must be non-empty IP addresses".to_string(),
        ));
    }
    if values.len() == 1 {
        Ok(json!(values[0]))
    } else {
        Ok(json!(values))
    }
}

pub(crate) fn apply_xray_routing_options(options: &mut XrayGenOptions, routing: &RoutingSettings) {
    options.routing = Some(XrayRoutingOptions {
        domain_strategy: routing.domain_strategy.clone(),
        direct: xray_route_list(&routing.direct),
        block: xray_route_list(&routing.block),
    });
}

fn xray_route_list(routes: &RouteList) -> XrayRouteList {
    XrayRouteList {
        domain: routes.domain.clone(),
        ip: routes.ip.clone(),
        geosite: routes.geosite.clone(),
        geoip: routes.geoip.clone(),
    }
}

pub(crate) fn build_singbox_routing_options(routing: &RoutingSettings) -> SingboxRoutingOptions {
    SingboxRoutingOptions {
        direct: singbox_route_list(&routing.direct),
        block: singbox_route_list(&routing.block),
    }
}

fn singbox_route_list(routes: &RouteList) -> SingboxRouteList {
    SingboxRouteList {
        domain: routes.domain.clone(),
        ip: routes.ip.clone(),
        geosite: routes.geosite.clone(),
        geoip: routes.geoip.clone(),
    }
}

/// Resolve the inbound listen address when `[runtime.network].listen_interface`
/// is set, returning the interface's address. Returns `None` when no interface
/// is configured so callers fall back to the per-inbound host.
pub(crate) fn resolve_listen_interface_addr(
    runtime: &RuntimeSettings,
) -> crate::app::Result<Option<String>> {
    let interface = runtime.network.listen_interface.trim();
    if interface.is_empty() {
        return Ok(None);
    }
    crate::support::net::interface_address(interface)
        .map(Some)
        .ok_or_else(|| {
            AppError::InvalidArgument(format!(
                "[runtime.network].listen_interface \"{interface}\" has no resolvable address"
            ))
        })
}

/// Translate the dual-form packets setting to Xray's `packets` value: the
/// `tlshello` keyword, or a `min-max` range when `packets_mode = "range"`.
fn fragment_packets(runtime: &RuntimeSettings) -> String {
    if runtime.fragment.packets_mode.trim() == "range" {
        format_range(runtime.fragment.packets)
    } else {
        "tlshello".to_string()
    }
}

fn format_range(range: [u32; 2]) -> String {
    format!("{}-{}", range[0], range[1])
}

fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn default_dns_settings_are_omitted_from_generated_options() {
        let dns = DnsSettings::default();
        let mut xray = XrayGenOptions::default();

        apply_xray_dns_options(&mut xray, &dns).expect("default Xray DNS should be accepted");

        assert!(xray.dns.is_none());
        assert!(
            build_singbox_dns_options(&dns)
                .expect("default sing-box DNS should be accepted")
                .is_none()
        );
    }

    #[test]
    fn xray_dns_mapping_preserves_documented_wire_fields() {
        let mut dns = DnsSettings {
            query_strategy: "UseIPv6".to_string(),
            servers: vec!["8.8.8.8".to_string()],
            use_system_hosts: false,
            disable_cache: true,
            disable_fallback: true,
            enable_parallel_query: false,
            ..Default::default()
        };
        dns.hosts.insert(
            "full:example.test".to_string(),
            DnsHostValue::Many(vec!["192.0.2.10".to_string(), "2001:db8::10".to_string()]),
        );

        let mut options = XrayGenOptions::default();
        apply_xray_dns_options(&mut options, &dns).expect("Xray DNS should map");
        let value = serde_json::to_value(options.dns.expect("DNS output")).unwrap();

        assert_eq!(value["queryStrategy"], "UseIPv6");
        assert_eq!(value["servers"], serde_json::json!(["8.8.8.8"]));
        assert_eq!(value["useSystemHosts"], false);
        assert_eq!(value["disableCache"], true);
        assert_eq!(value["disableFallback"], true);
        assert_eq!(value["enableParallelQuery"], false);
        assert_eq!(
            value["hosts"]["full:example.test"],
            serde_json::json!(["192.0.2.10", "2001:db8::10"])
        );
    }

    #[test]
    fn singbox_dns_mapping_uses_typed_servers_and_safe_fallbacks() {
        let mut dns = DnsSettings {
            query_strategy: "UseIPv4".to_string(),
            servers: vec![
                "8.8.8.8".to_string(),
                "https://dns.google/dns-query".to_string(),
            ],
            use_system_hosts: false,
            disable_cache: true,
            ..Default::default()
        };
        dns.hosts.insert(
            "full:example.test".to_string(),
            DnsHostValue::One("192.0.2.10".to_string()),
        );

        let output = build_singbox_dns_options(&dns)
            .expect("sing-box DNS should map")
            .expect("non-default DNS should be emitted");
        let value = serde_json::to_value(output).unwrap();

        assert_eq!(value["strategy"], "ipv4_only");
        assert_eq!(value["disable_cache"], true);
        assert_eq!(value["final"], "xrat-dns-0");
        assert_eq!(value["servers"][0]["type"], "udp");
        assert_eq!(value["servers"][1]["type"], "https");
        assert_eq!(value["servers"][1]["path"], "/dns-query");
        assert_eq!(
            value["servers"][1]["domain_resolver"],
            SINGBOX_LOCAL_DNS_TAG
        );
        assert_eq!(value["servers"][2]["type"], "local");
        assert_eq!(value["servers"][3]["type"], "hosts");
        assert_eq!(value["servers"][3]["path"], serde_json::json!([]));
        assert_eq!(
            value["rules"][0]["domain"],
            serde_json::json!(["example.test"])
        );
        assert_eq!(value["rules"][0]["action"], "route");
        assert_eq!(value["rules"][0]["server"], SINGBOX_HOSTS_DNS_TAG);
    }

    #[test]
    fn singbox_dns_mapping_rejects_unrepresentable_settings() {
        let mut dns = DnsSettings {
            servers: vec!["1.1.1.1".to_string()],
            ..Default::default()
        };
        let error =
            build_singbox_dns_options(&dns).expect_err("UseSystem must not be silently remapped");
        assert!(
            error
                .to_string()
                .contains("no exact modern sing-box equivalent")
        );

        dns.query_strategy = "UseIPv4".to_string();
        dns.disable_fallback = true;
        let error = build_singbox_dns_options(&dns).expect_err("fallback must be rejected");
        assert!(error.to_string().contains("disable_fallback"));
    }

    #[test]
    fn singbox_dns_mapping_rejects_advanced_hosts_and_unsupported_servers() {
        let mut hosts = BTreeMap::new();
        hosts.insert(
            "domain:example.test".to_string(),
            DnsHostValue::One("192.0.2.10".to_string()),
        );
        let dns = DnsSettings {
            query_strategy: "UseIPv4".to_string(),
            hosts,
            ..Default::default()
        };
        let error = build_singbox_dns_options(&dns).expect_err("domain: keys must be rejected");
        assert!(error.to_string().contains("not an exact hostname"));

        let dns = DnsSettings {
            query_strategy: "UseIPv4".to_string(),
            servers: vec!["h2c://dns.example/dns-query".to_string()],
            ..Default::default()
        };
        let error = build_singbox_dns_options(&dns).expect_err("h2c must be rejected");
        assert!(
            error
                .to_string()
                .contains("no safe modern sing-box mapping")
        );
    }
}
