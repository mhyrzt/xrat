use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, Response, header};

use crate::app::config::RoutingSettings;
use crate::server::{ServerError, ServerResult, ServerState};

/// Local proxy endpoints used to render a PAC file. Hosts/ports are non-secret
/// and safe to expose; Shadowsocks credentials are intentionally excluded.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PacEndpoints {
    pub http: Option<(String, u16)>,
    pub socks: Option<(String, u16)>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PacRules {
    pub direct_domains: Vec<String>,
    pub direct_cidrs: Vec<String>,
    pub block_domains: Vec<String>,
    pub block_cidrs: Vec<String>,
}

impl PacRules {
    pub fn from_routing(routing: &RoutingSettings) -> Self {
        Self {
            direct_domains: routing.direct.domain.clone(),
            direct_cidrs: routing.direct.ip.clone(),
            block_domains: routing.block.domain.clone(),
            block_cidrs: routing.block.ip.clone(),
        }
    }

    fn is_empty(&self) -> bool {
        self.direct_domains.is_empty()
            && self.direct_cidrs.is_empty()
            && self.block_domains.is_empty()
            && self.block_cidrs.is_empty()
    }
}

/// `GET /proxy.pac` — unauthenticated local helper. PAC consumers (browsers,
/// desktop proxy settings) usually cannot send auth headers, and the file only
/// exposes non-secret local endpoint data.
pub async fn proxy_pac(
    State(state): State<ServerState>,
    headers: HeaderMap,
) -> ServerResult<Response<Body>> {
    if !state.pac_enabled {
        return Err(ServerError::NotFound);
    }
    require_allowed_pac_host(&state, &headers)?;

    let endpoints = active_endpoints(&state).await?;
    let body = render_pac(&endpoints, &state.pac_rules);

    let mut response = Response::new(Body::from(body));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/x-ns-proxy-autoconfig"),
    );
    Ok(response)
}

fn require_allowed_pac_host(state: &ServerState, headers: &HeaderMap) -> ServerResult<()> {
    let Some(host) = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .and_then(host_without_port)
    else {
        return Err(ServerError::PacHostNotAllowed);
    };

    let allowed = state
        .pac_allowed_hosts
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(host));
    if allowed {
        Ok(())
    } else {
        Err(ServerError::PacHostNotAllowed)
    }
}

fn host_without_port(value: &str) -> Option<&str> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Some(rest) = value.strip_prefix('[') {
        return rest.split_once(']').map(|(host, _)| host);
    }
    Some(value.split_once(':').map_or(value, |(host, _)| host))
}

async fn active_endpoints(state: &ServerState) -> ServerResult<PacEndpoints> {
    let Some(session) = state.db.get_running_runtime_session().await? else {
        return Ok(PacEndpoints::default());
    };

    let http = match (session.http_host, session.http_port) {
        (Some(host), Some(port)) if port > 0 && port <= i64::from(u16::MAX) => {
            Some((host, port as u16))
        }
        _ => None,
    };
    let socks = match (session.socks_host, session.socks_port) {
        (Some(host), Some(port)) if port > 0 && port <= i64::from(u16::MAX) => {
            Some((host, port as u16))
        }
        _ => None,
    };

    Ok(PacEndpoints { http, socks })
}

/// Render a deterministic PAC file. Local and private destinations bypass the
/// proxy; everything else prefers SOCKS, then HTTP, then `DIRECT`. With no
/// active proxy the file routes everything `DIRECT`.
pub fn render_pac(endpoints: &PacEndpoints, rules: &PacRules) -> String {
    let mut chain: Vec<String> = Vec::new();
    if let Some((host, port)) = &endpoints.socks {
        chain.push(format!("SOCKS5 {host}:{port}"));
    }
    if let Some((host, port)) = &endpoints.http {
        chain.push(format!("PROXY {host}:{port}"));
    }
    chain.push("DIRECT".to_string());
    let proxy_chain = chain.join("; ");

    if !rules.is_empty() {
        return render_pac_with_rules(&proxy_chain, rules);
    }

    format!(
        "function FindProxyForURL(url, host) {{\n\
\x20 if (\n\
\x20   isPlainHostName(host) ||\n\
\x20   shExpMatch(host, \"*.local\") ||\n\
\x20   host == \"localhost\" ||\n\
\x20   host == \"127.0.0.1\" ||\n\
\x20   host == \"::1\" ||\n\
\x20   isInNet(host, \"10.0.0.0\", \"255.0.0.0\") ||\n\
\x20   isInNet(host, \"172.16.0.0\", \"255.240.0.0\") ||\n\
\x20   isInNet(host, \"192.168.0.0\", \"255.255.0.0\")\n\
\x20 ) {{\n\
\x20   return \"DIRECT\";\n\
\x20 }}\n\
\x20 return \"{proxy_chain}\";\n\
}}\n"
    )
}

fn render_pac_with_rules(proxy_chain: &str, rules: &PacRules) -> String {
    let mut pac = String::from(
        "function FindProxyForURL(url, host) {\n\
  if (\n\
    isPlainHostName(host) ||\n\
    shExpMatch(host, \"*.local\") ||\n\
    host == \"localhost\" ||\n\
    host == \"127.0.0.1\" ||\n\
    host == \"::1\" ||\n\
    isInNet(host, \"10.0.0.0\", \"255.0.0.0\") ||\n\
    isInNet(host, \"172.16.0.0\", \"255.240.0.0\") ||\n\
    isInNet(host, \"192.168.0.0\", \"255.255.0.0\")\n\
  ) {\n\
    return \"DIRECT\";\n\
  }\n",
    );

    append_rule_block(
        &mut pac,
        &rules.direct_domains,
        &rules.direct_cidrs,
        "DIRECT",
    );
    append_rule_block(
        &mut pac,
        &rules.block_domains,
        &rules.block_cidrs,
        "PROXY 127.0.0.1:9",
    );
    pac.push_str(&format!("  return \"{}\";\n}}\n", escape_js(proxy_chain)));
    pac
}

fn append_rule_block(pac: &mut String, domains: &[String], cidrs: &[String], action: &str) {
    let mut conditions = Vec::new();
    for domain in domains {
        if let Some(condition) = domain_condition(domain) {
            conditions.push(condition);
        }
    }
    for cidr in cidrs {
        if let Some(condition) = cidr_condition(cidr) {
            conditions.push(condition);
        }
    }
    if conditions.is_empty() {
        return;
    }

    pac.push_str("  if (\n");
    for (index, condition) in conditions.iter().enumerate() {
        let suffix = if index + 1 == conditions.len() {
            "\n"
        } else {
            " ||\n"
        };
        pac.push_str(&format!("    {condition}{suffix}"));
    }
    pac.push_str("  ) {\n");
    pac.push_str(&format!("    return \"{}\";\n", escape_js(action)));
    pac.push_str("  }\n");
}

fn domain_condition(domain: &str) -> Option<String> {
    let domain = domain.trim();
    if domain.is_empty() || domain.starts_with("geosite:") || domain.starts_with("regexp:") {
        return None;
    }
    if let Some(domain) = domain.strip_prefix("full:") {
        let domain = domain.trim_start_matches('.');
        if domain.is_empty() {
            return None;
        }
        return Some(format!("host == \"{}\"", escape_js(domain)));
    }
    let domain = domain.strip_prefix("domain:").unwrap_or(domain);
    let domain = domain.trim_start_matches('.');
    if domain.is_empty() {
        return None;
    }

    if domain.contains('*') || domain.contains('?') {
        return Some(format!("shExpMatch(host, \"{}\")", escape_js(domain)));
    }
    Some(format!(
        "(host == \"{}\" || shExpMatch(host, \"*.{}\"))",
        escape_js(domain),
        escape_js(domain)
    ))
}

fn cidr_condition(cidr: &str) -> Option<String> {
    let (ip, prefix) = cidr.trim().split_once('/')?;
    let ip: std::net::Ipv4Addr = ip.parse().ok()?;
    let prefix: u32 = prefix.parse().ok()?;
    if prefix > 32 {
        return None;
    }
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    let network = u32::from(ip) & mask;
    Some(format!(
        "isInNet(host, \"{}\", \"{}\")",
        std::net::Ipv4Addr::from(network),
        std::net::Ipv4Addr::from(mask)
    ))
}

fn escape_js(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_socks_preferred_chain() {
        let pac = render_pac(
            &PacEndpoints {
                http: Some(("127.0.0.1".to_string(), 18201)),
                socks: Some(("127.0.0.1".to_string(), 18200)),
            },
            &PacRules::default(),
        );
        assert!(pac.contains("return \"SOCKS5 127.0.0.1:18200; PROXY 127.0.0.1:18201; DIRECT\";"));
        assert!(pac.contains("isPlainHostName(host)"));
    }

    #[test]
    fn renders_http_only_chain() {
        let pac = render_pac(
            &PacEndpoints {
                http: Some(("127.0.0.1".to_string(), 18201)),
                socks: None,
            },
            &PacRules::default(),
        );
        assert!(pac.contains("return \"PROXY 127.0.0.1:18201; DIRECT\";"));
    }

    #[test]
    fn renders_socks_only_chain() {
        let pac = render_pac(
            &PacEndpoints {
                http: None,
                socks: Some(("127.0.0.1".to_string(), 18200)),
            },
            &PacRules::default(),
        );
        assert!(pac.contains("return \"SOCKS5 127.0.0.1:18200; DIRECT\";"));
    }

    #[test]
    fn renders_direct_when_no_proxy_active() {
        let pac = render_pac(&PacEndpoints::default(), &PacRules::default());
        assert!(pac.contains("return \"DIRECT\";"));
    }

    #[test]
    fn renders_direct_domain_rule() {
        let pac = render_pac(
            &PacEndpoints::default(),
            &PacRules {
                direct_domains: vec!["example.com".to_string()],
                ..PacRules::default()
            },
        );

        assert!(pac.contains("host == \"example.com\""));
        assert!(pac.contains("shExpMatch(host, \"*.example.com\")"));
        assert!(pac.contains("return \"DIRECT\";"));
    }

    #[test]
    fn renders_cidr_rule_with_mask() {
        let pac = render_pac(
            &PacEndpoints::default(),
            &PacRules {
                direct_cidrs: vec!["203.0.113.9/24".to_string()],
                ..PacRules::default()
            },
        );

        assert!(pac.contains("isInNet(host, \"203.0.113.0\", \"255.255.255.0\")"));
    }

    #[test]
    fn renders_block_rule_before_default_proxy() {
        let pac = render_pac(
            &PacEndpoints {
                http: Some(("127.0.0.1".to_string(), 18201)),
                socks: None,
            },
            &PacRules {
                block_domains: vec!["blocked.example".to_string()],
                ..PacRules::default()
            },
        );

        let block_index = pac
            .find("blocked.example")
            .expect("block rule should render");
        let default_index = pac
            .find("return \"PROXY 127.0.0.1:18201; DIRECT\";")
            .expect("default proxy fallback should render");
        assert!(block_index < default_index);
        assert!(pac.contains("return \"PROXY 127.0.0.1:9\";"));
    }

    #[test]
    fn extracts_host_without_port() {
        assert_eq!(host_without_port("localhost:18203"), Some("localhost"));
        assert_eq!(host_without_port("127.0.0.1:18203"), Some("127.0.0.1"));
        assert_eq!(host_without_port("[::1]:18203"), Some("::1"));
    }
}
