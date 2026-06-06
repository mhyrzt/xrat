use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderValue, Response, header};

use crate::server::{ServerResult, ServerState};

/// Local proxy endpoints used to render a PAC file. Hosts/ports are non-secret
/// and safe to expose; Shadowsocks credentials are intentionally excluded.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PacEndpoints {
    pub http: Option<(String, u16)>,
    pub socks: Option<(String, u16)>,
}

/// `GET /proxy.pac` — unauthenticated local helper. PAC consumers (browsers,
/// desktop proxy settings) usually cannot send auth headers, and the file only
/// exposes non-secret local endpoint data.
pub async fn proxy_pac(State(state): State<ServerState>) -> ServerResult<Response<Body>> {
    let endpoints = active_endpoints(&state).await?;
    let body = render_pac(&endpoints);

    let mut response = Response::new(Body::from(body));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/x-ns-proxy-autoconfig"),
    );
    Ok(response)
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
pub fn render_pac(endpoints: &PacEndpoints) -> String {
    let mut chain: Vec<String> = Vec::new();
    if let Some((host, port)) = &endpoints.socks {
        chain.push(format!("SOCKS5 {host}:{port}"));
    }
    if let Some((host, port)) = &endpoints.http {
        chain.push(format!("PROXY {host}:{port}"));
    }
    chain.push("DIRECT".to_string());
    let proxy_chain = chain.join("; ");

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_socks_preferred_chain() {
        let pac = render_pac(&PacEndpoints {
            http: Some(("127.0.0.1".to_string(), 18201)),
            socks: Some(("127.0.0.1".to_string(), 18200)),
        });
        assert!(pac.contains("return \"SOCKS5 127.0.0.1:18200; PROXY 127.0.0.1:18201; DIRECT\";"));
        assert!(pac.contains("isPlainHostName(host)"));
    }

    #[test]
    fn renders_http_only_chain() {
        let pac = render_pac(&PacEndpoints {
            http: Some(("127.0.0.1".to_string(), 18201)),
            socks: None,
        });
        assert!(pac.contains("return \"PROXY 127.0.0.1:18201; DIRECT\";"));
    }

    #[test]
    fn renders_socks_only_chain() {
        let pac = render_pac(&PacEndpoints {
            http: None,
            socks: Some(("127.0.0.1".to_string(), 18200)),
        });
        assert!(pac.contains("return \"SOCKS5 127.0.0.1:18200; DIRECT\";"));
    }

    #[test]
    fn renders_direct_when_no_proxy_active() {
        let pac = render_pac(&PacEndpoints::default());
        assert!(pac.contains("return \"DIRECT\";"));
    }
}
