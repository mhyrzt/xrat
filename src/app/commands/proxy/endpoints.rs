use crate::app::commands::output;
use crate::app::commands::runtime_output::format_inbound_endpoint;
use crate::app::context::AppContext;

use super::{http_proxy_url, loopback_host, resolve_active_endpoints, socks_proxy_url};

pub(super) async fn run(context: &AppContext, json: bool) -> crate::app::Result<()> {
    let active = resolve_active_endpoints(context).await?;

    let mut rows: Vec<(&str, String)> = Vec::new();
    if let Some((host, port)) = &active.http {
        rows.push((
            "HTTP",
            format!("http://{}", format_inbound_endpoint(host, *port)),
        ));
    }
    if let Some((host, port)) = &active.socks {
        rows.push((
            "SOCKS5",
            format!("socks5://{}", format_inbound_endpoint(host, *port)),
        ));
    }
    if let Some((host, port)) = &active.shadowsocks {
        // The runtime session does not persist Shadowsocks credentials, so emit
        // the endpoint without a leaky partial ss:// URI.
        rows.push((
            "Shadowsocks",
            format!(
                "{} (credentials not shown)",
                format_inbound_endpoint(host, *port)
            ),
        ));
    }

    let server = &context.app_config.server;
    let pac_url = (server.enabled && server.pac_enabled).then(|| {
        let host = if server.host == "0.0.0.0" || server.host.is_empty() {
            "127.0.0.1"
        } else {
            server.host.as_str()
        };
        format!("http://{host}:{}/proxy.pac", server.port)
    });
    if let Some(url) = &pac_url {
        rows.push(("PAC URL", url.clone()));
    }

    let http_proxy = active
        .http
        .as_ref()
        .map(|(host, port)| http_proxy_url(host, *port))
        .or_else(|| {
            active
                .socks
                .as_ref()
                .map(|(host, port)| socks_proxy_url(host, *port))
        });
    let all_proxy = active
        .socks
        .as_ref()
        .map(|(host, port)| socks_proxy_url(host, *port))
        .or_else(|| {
            active
                .http
                .as_ref()
                .map(|(host, port)| http_proxy_url(host, *port))
        });

    if json {
        let payload = serde_json::json!({
            "endpoints": {
                "http": active.http.as_ref().map(|(h, p)| format!("http://{}:{p}", loopback_host(h))),
                "socks5": active.socks.as_ref().map(|(h, p)| format!("socks5://{}:{p}", loopback_host(h))),
                "shadowsocks": active.shadowsocks.as_ref().map(|(h, p)| format!("{}:{p}", loopback_host(h))),
                "pac": pac_url,
            },
            "environment": {
                "http_proxy": http_proxy.clone(),
                "https_proxy": http_proxy.clone(),
                "all_proxy": all_proxy.clone(),
            },
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    if rows.is_empty() {
        println!(
            "{}",
            output::empty_message(
                "No active proxy endpoints. Start a runtime with `xrat connect <id>`."
            )
        );
        return Ok(());
    }

    if let Some(value) = &http_proxy {
        rows.push(("http_proxy", value.clone()));
        rows.push(("https_proxy", value.clone()));
    }
    if let Some(value) = &all_proxy {
        rows.push(("all_proxy", value.clone()));
    }
    rows.push(("toggle", "eval \"$(xrat proxy toggle)\"".to_string()));

    let kv: Vec<(&str, String)> = rows;
    println!(
        "{}",
        output::format_kv(Some("Proxy info"), &kv, output::color_enabled())
    );
    Ok(())
}
