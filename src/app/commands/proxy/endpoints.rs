use crate::app::commands::output;
use crate::app::commands::runtime_output::format_inbound_endpoint;
use crate::app::context::AppContext;

use super::resolve_active_endpoints;

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
    if server.enabled {
        let host = if server.host == "0.0.0.0" || server.host.is_empty() {
            "127.0.0.1"
        } else {
            server.host.as_str()
        };
        rows.push(("PAC", format!("http://{host}:{}/proxy.pac", server.port)));
    }

    if json {
        let payload = serde_json::json!({
            "http": active.http.as_ref().map(|(h, p)| format!("http://{h}:{p}")),
            "socks5": active.socks.as_ref().map(|(h, p)| format!("socks5://{h}:{p}")),
            "shadowsocks": active.shadowsocks.as_ref().map(|(h, p)| format!("{h}:{p}")),
            "pac": server.enabled.then(|| {
                let host = if server.host == "0.0.0.0" || server.host.is_empty() {
                    "127.0.0.1"
                } else {
                    server.host.as_str()
                };
                format!("http://{host}:{}/proxy.pac", server.port)
            }),
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

    let kv: Vec<(&str, String)> = rows;
    println!(
        "{}",
        output::format_kv(Some("Proxy endpoints"), &kv, output::color_enabled())
    );
    Ok(())
}
