use super::*;

pub fn generate_probe_config(node: &Node, local_port: u16) -> Result<XrayConfig, String> {
    generate_probe_config_with_options(node, local_port, &XrayGenOptions::default())
}

pub fn generate_probe_config_with_options(
    node: &Node,
    local_port: u16,
    options: &XrayGenOptions,
) -> Result<XrayConfig, String> {
    let inbound = Inbound {
        sniffing: None,
        tag: "probe-in".to_string(),
        port: Some(local_port),
        listen: Some("127.0.0.1".to_string()),
        protocol: "socks".to_string(),
        settings: Some(json!({"udp": false})),
    };

    let outbound = node_to_outbound(node, "proxy", options.compatibility)?;

    let mut config = XrayConfig {
        log: LogConfig {
            loglevel: "warning".to_string(),
        },
        inbounds: vec![inbound],
        outbounds: vec![outbound],
        dns: None,
        api: None,
        stats: None,
        policy: None,
        routing: None,
        fake_dns: None,
    };
    apply_runtime_tuning(&mut config, options);
    Ok(config)
}

pub fn generate_runtime_config(
    node: &Node,
    socks_port: u16,
    http_port: Option<u16>,
) -> Result<XrayConfig, String> {
    generate_runtime_config_with_inbounds(node, "127.0.0.1", socks_port, None, http_port)
}

pub fn generate_runtime_config_with_inbounds(
    node: &Node,
    socks_host: &str,
    socks_port: u16,
    http_host: Option<&str>,
    http_port: Option<u16>,
) -> Result<XrayConfig, String> {
    generate_runtime_config_for_inbounds(
        node,
        Some((socks_host, socks_port, true)),
        http_port.map(|port| (http_host.unwrap_or(socks_host), port)),
    )
}

pub fn generate_runtime_config_for_inbounds(
    node: &Node,
    socks: Option<(&str, u16, bool)>,
    http: Option<(&str, u16)>,
) -> Result<XrayConfig, String> {
    generate_runtime_config_for_inbounds_with_options(node, socks, http, &XrayGenOptions::default())
}

pub fn generate_runtime_config_for_inbounds_with_options(
    node: &Node,
    socks: Option<(&str, u16, bool)>,
    http: Option<(&str, u16)>,
    options: &XrayGenOptions,
) -> Result<XrayConfig, String> {
    let inbounds = build_inbounds(socks, http);
    let outbound = node_to_outbound(node, "proxy", options.compatibility)?;

    let mut config = XrayConfig {
        log: LogConfig {
            loglevel: "warning".to_string(),
        },
        inbounds,
        outbounds: vec![outbound],
        dns: None,
        api: None,
        stats: None,
        policy: None,
        routing: None,
        fake_dns: None,
    };
    apply_runtime_routing(&mut config, options.routing.as_ref());
    apply_runtime_tuning(&mut config, options);
    Ok(config)
}

/// Enable the xray gRPC StatsService on an already-built runtime config. Adds a
/// `dokodemo-door` inbound tagged `api`, the `api`/`stats`/`policy` objects, and
/// a routing rule that dispatches the api inbound to the api handler. Counters
/// are enabled for system inbound/outbound traffic so totals are available.
pub fn enable_stats_api(config: &mut XrayConfig, host: &str, port: u16) {
    use xrat_config::parsing::core::{ApiObject, ApiServiceName, PolicyObject, SystemPolicyObject};

    config.inbounds.push(Inbound {
        sniffing: None,
        tag: "api".to_string(),
        port: Some(port),
        listen: Some(host.to_string()),
        protocol: "dokodemo-door".to_string(),
        settings: Some(json!({ "address": host })),
    });
    config.api = Some(ApiObject {
        tag: "api".to_string(),
        listen: None,
        services: vec![ApiServiceName::StatsService],
    });
    config.stats = Some(json!({}));
    config.policy = Some(PolicyObject {
        levels: None,
        system: Some(SystemPolicyObject {
            stats_inbound_uplink: Some(true),
            stats_inbound_downlink: Some(true),
            stats_outbound_uplink: Some(true),
            stats_outbound_downlink: Some(true),
        }),
    });
    let routing = config
        .routing
        .get_or_insert_with(|| super::super::RoutingConfig {
            domain_strategy: None,
            rules: Vec::new(),
        });
    routing.rules.insert(
        0,
        field_rule(None, None, Some(vec!["api".to_string()]), "api"),
    );
}

#[derive(Debug, Clone)]
pub struct XrayTunCaptureOptions<'a> {
    pub interface_name: &'a str,
    pub mtu: u32,
    pub address: &'a [String],
    pub auto_route: bool,
    pub resolved_hosts: &'a [(String, String)],
}

/// Enable TUN capture on an already-built Xray runtime config:
/// - adds a `tun` inbound tagged `tun-in` with routing/interface settings
/// - adds a `dns` outbound tagged `dns-out`
/// - inserts a priority routing rule diverting port 53 traffic from `tun-in` to `dns-out`
/// - ensures private/LAN IP ranges route `direct` to maintain local connectivity
/// - ensures DNS servers are configured (falling back to DoH over IP) and populates `dns.hosts`
pub fn enable_tun_capture(config: &mut XrayConfig, options: &XrayTunCaptureOptions<'_>) {
    use crate::xray::config::{
        Outbound, RoutingConfig, RoutingRule, XrayDnsConfig, XrayDnsHostValue,
    };

    let mut tun_settings = json!({
        "name": options.interface_name,
        "mtu": options.mtu,
        "gateway": options.address,
    });
    if options.auto_route {
        let routes = xray_tun_routes(options.address);
        tun_settings["autoSystemRoutingTable"] = json!(routes);
        tun_settings["autoOutboundsInterface"] = json!("auto");
    }
    config.inbounds.push(Inbound {
        sniffing: None,
        tag: "tun-in".to_string(),
        port: None,
        listen: None,
        protocol: "tun".to_string(),
        settings: Some(tun_settings),
    });

    if !config
        .outbounds
        .iter()
        .any(|outbound| outbound.tag == "dns-out")
    {
        config.outbounds.push(Outbound {
            tag: "dns-out".to_string(),
            protocol: "dns".to_string(),
            settings: json!({}),
            stream_settings: None,
            mux: None,
        });
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
    routing.rules.insert(
        0,
        RoutingRule {
            kind: "field".to_string(),
            domain: None,
            ip: None,
            port: Some("53".to_string()),
            network: Some("tcp,udp".to_string()),
            inbound_tag: Some(vec!["tun-in".to_string()]),
            process: None,
            outbound_tag: "dns-out".to_string(),
        },
    );

    let private_ips = vec![
        "10.0.0.0/8".to_string(),
        "172.16.0.0/12".to_string(),
        "192.168.0.0/16".to_string(),
        "127.0.0.0/8".to_string(),
        "fc00::/7".to_string(),
        "fe80::/10".to_string(),
    ];
    let has_private_direct_rule = routing.rules.iter().any(|rule| {
        rule.outbound_tag == "direct"
            && rule.ip.as_ref().is_some_and(|ips| {
                ips.iter()
                    .any(|ip| ip == "10.0.0.0/8" || ip == "geoip:private")
            })
    });
    if !has_private_direct_rule {
        routing.rules.push(RoutingRule {
            kind: "field".to_string(),
            domain: None,
            ip: Some(private_ips),
            port: None,
            network: None,
            inbound_tag: None,
            process: None,
            outbound_tag: "direct".to_string(),
        });
    }

    let dns = config.dns.get_or_insert_with(|| XrayDnsConfig {
        servers: vec![
            json!("https://1.1.1.1/dns-query"),
            json!("https://8.8.8.8/dns-query"),
        ],
        hosts: std::collections::BTreeMap::new(),
        query_strategy: "UseIPv4".to_string(),
        use_system_hosts: true,
        disable_cache: false,
        disable_fallback: false,
        disable_fallback_if_match: None,
        enable_parallel_query: true,
        tag: None,
    });
    if dns.servers.is_empty() {
        dns.servers = vec![
            json!("https://1.1.1.1/dns-query"),
            json!("https://8.8.8.8/dns-query"),
        ];
    }
    for (domain, ip) in options.resolved_hosts {
        dns.hosts
            .insert(domain.clone(), XrayDnsHostValue::One(ip.clone()));
    }
}

fn xray_tun_routes(address: &[String]) -> Vec<&'static str> {
    let mut routes = Vec::new();
    let mut has_ipv4 = false;
    let mut has_ipv6 = false;

    for addr in address {
        let ip_part = addr.split('/').next().unwrap_or(addr).trim();
        if let Ok(ip) = ip_part.parse::<std::net::IpAddr>() {
            match ip {
                std::net::IpAddr::V4(_) => has_ipv4 = true,
                std::net::IpAddr::V6(_) => has_ipv6 = true,
            }
        }
    }

    if has_ipv4 {
        routes.extend(["0.0.0.0/1", "128.0.0.0/1"]);
    }
    if has_ipv6 {
        routes.extend(["::/1", "8000::/1"]);
    }
    if routes.is_empty() {
        routes.extend(["0.0.0.0/1", "128.0.0.0/1"]);
    }
    routes
}

pub(super) fn build_inbounds(
    socks: Option<(&str, u16, bool)>,
    http: Option<(&str, u16)>,
) -> Vec<Inbound> {
    let mut inbounds = Vec::new();

    if let Some((host, port, udp)) = socks {
        inbounds.push(Inbound {
            sniffing: None,
            tag: "socks-in".to_string(),
            port: Some(port),
            listen: Some(host.to_string()),
            protocol: "socks".to_string(),
            settings: Some(json!({"udp": udp})),
        });
    }

    if let Some((host, port)) = http {
        inbounds.push(Inbound {
            sniffing: None,
            tag: "http-in".to_string(),
            port: Some(port),
            listen: Some(host.to_string()),
            protocol: "http".to_string(),
            settings: None,
        });
    }

    inbounds
}
