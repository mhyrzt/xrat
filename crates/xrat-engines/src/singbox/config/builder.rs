use super::*;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverse_mapping: Option<bool>,
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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SingboxInbound {
    Direct {
        tag: String,
        listen: String,
        listen_port: u16,
        override_address: String,
        override_port: u16,
    },
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
    Tun {
        tag: String,
        interface_name: String,
        address: Vec<String>,
        mtu: u32,
        stack: String,
        auto_route: bool,
        strict_route: bool,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        route_exclude_address: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub struct SingboxTunOptions {
    pub tag: String,
    pub interface_name: String,
    pub address: Vec<String>,
    pub mtu: u32,
    pub stack: String,
    pub auto_route: bool,
    pub strict_route: bool,
    pub route_exclude_address: Vec<String>,
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

    pub fn tun(options: SingboxTunOptions) -> Result<Self, String> {
        let interface_name = options.interface_name.trim().to_string();
        if interface_name.is_empty() {
            return Err("sing-box TUN inbound requires a non-empty interface_name".to_string());
        }
        if !matches!(options.stack.as_str(), "system" | "gvisor" | "mixed") {
            return Err(format!(
                "sing-box TUN stack must be system, gvisor, or mixed; got {:?}",
                options.stack
            ));
        }
        if !(1280..=65535).contains(&options.mtu) {
            return Err(format!(
                "sing-box TUN mtu must be in 1280..=65535; got {}",
                options.mtu
            ));
        }
        if options.address.is_empty() {
            return Err("sing-box TUN inbound requires at least one address CIDR".to_string());
        }
        for address in &options.address {
            if parse_cidr(address).is_none() {
                return Err(format!(
                    "sing-box TUN address {address:?} is not a valid CIDR"
                ));
            }
        }
        for address in &options.route_exclude_address {
            if parse_cidr(address).is_none() {
                return Err(format!(
                    "sing-box TUN route_exclude_address {address:?} is not a valid CIDR"
                ));
            }
        }
        Ok(Self::Tun {
            tag: options.tag,
            interface_name,
            address: options.address,
            mtu: options.mtu,
            stack: options.stack,
            auto_route: options.auto_route,
            strict_route: options.strict_route,
            route_exclude_address: options.route_exclude_address,
        })
    }
}

fn parse_cidr(value: &str) -> Option<(std::net::IpAddr, u8)> {
    let (address, prefix) = value.trim().split_once('/')?;
    let address = address.parse::<std::net::IpAddr>().ok()?;
    let prefix = prefix.parse::<u8>().ok()?;
    let max = if address.is_ipv4() { 32 } else { 128 };
    (prefix <= max).then_some((address, prefix))
}

pub(super) const SHADOWSOCKS_METHODS: &[&str] = &[
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_fakeip: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_id: Option<String>,
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
            auto_detect_interface: None,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SingboxTunSplitMode {
    #[default]
    All,
    Blacklist,
    Whitelist,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SingboxTunSplitOptions {
    pub mode: SingboxTunSplitMode,
    pub process_name: Vec<String>,
    pub process_path: Vec<String>,
    pub process_path_regex: Vec<String>,
}

impl SingboxTunSplitOptions {
    pub fn has_matchers(&self) -> bool {
        !self.process_name.is_empty()
            || !self.process_path.is_empty()
            || !self.process_path_regex.is_empty()
    }
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
            store_fakeip: None,
            cache_id: None,
        });
    }

    /// Shape the generated route for a TUN inbound: bind outbound dials to the
    /// detected physical interface to avoid routing loops, and send private/LAN
    /// destinations direct so local networks stay reachable.
    pub fn enable_tun_route(&mut self) {
        self.enable_tun_route_with_split(&SingboxTunSplitOptions::default());
    }

    /// Shape the generated route for a TUN inbound with optional per-application
    /// split tunneling rules (`all`, `blacklist`, or `whitelist`) scoped to `tun-in`.
    pub fn enable_tun_route_with_split(&mut self, split: &SingboxTunSplitOptions) {
        let route = self.route.get_or_insert_with(|| SingboxRoute {
            rules: Vec::new(),
            rule_set: Vec::new(),
            final_outbound: "proxy".to_string(),
            default_domain_resolver: None,
            auto_detect_interface: None,
        });
        route.auto_detect_interface = Some(true);
        if !route.rules.iter().any(|rule| rule["action"] == "sniff") {
            route
                .rules
                .insert(0, serde_json::json!({"action": "sniff"}));
        }
        let sniff_after = route
            .rules
            .iter()
            .position(|rule| rule["action"] == "sniff")
            .map_or(0, |idx| idx + 1);
        if !route.rules.iter().any(|rule| rule["ip_is_private"] == true) {
            route.rules.insert(
                sniff_after,
                serde_json::json!({
                    "ip_is_private": true,
                    "action": "route",
                    "outbound": "direct",
                }),
            );
        }
        let split_insert_idx = route
            .rules
            .iter()
            .position(|rule| rule["ip_is_private"] == true)
            .map_or(sniff_after, |idx| idx + 1);

        let mut split_rules = Vec::new();
        match split.mode {
            SingboxTunSplitMode::All => {}
            SingboxTunSplitMode::Blacklist => {
                if let Some(rule) = build_singbox_split_rule(split, "direct") {
                    split_rules.push(rule);
                }
            }
            SingboxTunSplitMode::Whitelist => {
                if let Some(rule) = build_singbox_split_rule(split, "proxy") {
                    split_rules.push(rule);
                }
                split_rules.push(serde_json::json!({
                    "inbound": ["tun-in"],
                    "action": "route",
                    "outbound": "direct",
                }));
            }
        }
        if !split_rules.is_empty() {
            route
                .rules
                .splice(split_insert_idx..split_insert_idx, split_rules);
        }
    }
}

fn build_singbox_split_rule(
    split: &SingboxTunSplitOptions,
    outbound: &str,
) -> Option<serde_json::Value> {
    if !split.has_matchers() {
        return None;
    }
    let mut object = serde_json::Map::new();
    object.insert("inbound".to_string(), serde_json::json!(["tun-in"]));
    insert_non_empty(&mut object, "process_name", split.process_name.clone());
    insert_non_empty(&mut object, "process_path", split.process_path.clone());
    insert_non_empty(
        &mut object,
        "process_path_regex",
        split.process_path_regex.clone(),
    );
    object.insert("action".to_string(), serde_json::json!("route"));
    object.insert("outbound".to_string(), serde_json::json!(outbound));
    Some(serde_json::Value::Object(object))
}

pub(super) fn build_route(
    routing: Option<&SingboxRoutingOptions>,
) -> Result<Option<SingboxRoute>, String> {
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
        auto_detect_interface: None,
    }))
}

pub(super) fn append_route_rules(
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

pub(super) const GEOSITE_RULE_SET_BASE: &str =
    "https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set";

pub(super) const GEOIP_RULE_SET_BASE: &str =
    "https://raw.githubusercontent.com/SagerNet/sing-geoip/rule-set";

/// Register remote SagerNet rule-sets for the configured geosite/geoip
/// categories and return their tags for the referencing route rule.
pub(super) fn append_rule_set_rules(
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

pub(super) fn validate_rule_set_name(kind: &str, name: &str, field: &str) -> Result<(), String> {
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

pub(super) fn insert_non_empty(
    object: &mut serde_json::Map<String, serde_json::Value>,
    field: &str,
    values: Vec<String>,
) {
    if !values.is_empty() {
        object.insert(field.to_string(), serde_json::json!(values));
    }
}
