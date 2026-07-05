//! Runtime outbound tuning applied on top of a generated Xray config: client
//! Mux, TCP fragmentation, and outbound interface/fwmark binding. These are
//! global runtime options sourced from `config.toml`, not per-node data, so they
//! are applied as a post-build mutation of the generated config rather than
//! threaded through node-to-outbound conversion.

use serde_json::json;

use super::types::{Mux, Outbound, Sockopt, StreamSettings, XrayConfig};

const FRAGMENT_TAG: &str = "fragment";

/// Resolved outbound tuning for a single generated config. Empty by default so
/// the generated config is unchanged unless the user opts in.
#[derive(Debug, Clone, Default)]
pub struct XrayGenOptions {
    pub mux: Option<MuxOptions>,
    pub fragment: Option<FragmentOptions>,
    pub interface: Option<String>,
    pub mark: Option<i64>,
    pub bind_address: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MuxOptions {
    pub concurrency: i32,
    pub xudp_concurrency: i32,
    pub xudp_proxy_udp443: String,
}

#[derive(Debug, Clone)]
pub struct FragmentOptions {
    pub packets: String,
    pub length: String,
    pub interval: String,
}

impl XrayGenOptions {
    fn is_noop(&self) -> bool {
        self.mux.is_none()
            && self.fragment.is_none()
            && self.interface.is_none()
            && self.mark.is_none()
    }
}

/// Apply runtime outbound tuning to an already-built runtime/probe config. The
/// proxy outbound is always `outbounds[0]` by construction. `bind_address` is
/// intentionally not applied here: Xray's `sockopt` has no source-IP field, so
/// it is validated and warned about at the call site instead of silently
/// emitted.
pub fn apply_runtime_tuning(config: &mut XrayConfig, options: &XrayGenOptions) {
    if options.is_noop() {
        return;
    }

    if let (Some(mux), Some(proxy)) = (&options.mux, config.outbounds.first_mut()) {
        proxy.mux = Some(Mux {
            enabled: true,
            concurrency: mux.concurrency,
            xudp_concurrency: mux.xudp_concurrency,
            xudp_proxy_udp443: mux.xudp_proxy_udp443.clone(),
        });
    }

    let bind = (options.interface.clone(), options.mark);
    let has_bind = bind.0.is_some() || bind.1.is_some();

    if let Some(fragment) = &options.fragment {
        let mut fragment_out = build_fragment_outbound(fragment);
        // The fragment freedom outbound performs the actual egress dial, so
        // interface/fwmark binding belongs on it rather than the proxy outbound.
        if has_bind {
            apply_sockopt(stream_settings_mut(&mut fragment_out), bind.0, bind.1, None);
        }
        config.outbounds.push(fragment_out);

        if let Some(proxy) = config.outbounds.first_mut() {
            apply_sockopt(
                stream_settings_mut(proxy),
                None,
                None,
                Some(FRAGMENT_TAG.to_string()),
            );
        }
    } else if has_bind && let Some(proxy) = config.outbounds.first_mut() {
        apply_sockopt(stream_settings_mut(proxy), bind.0, bind.1, None);
    }
}

fn build_fragment_outbound(fragment: &FragmentOptions) -> Outbound {
    Outbound {
        tag: FRAGMENT_TAG.to_string(),
        protocol: "freedom".to_string(),
        settings: json!({
            "domainStrategy": "AsIs",
            "fragment": {
                "packets": fragment.packets,
                "length": fragment.length,
                "interval": fragment.interval,
            }
        }),
        stream_settings: None,
        mux: None,
    }
}

/// Borrow the outbound's stream settings, creating a minimal `tcp` block when
/// the upstream protocol (socks/http) produced none, so a `sockopt` can attach.
fn stream_settings_mut(outbound: &mut Outbound) -> &mut StreamSettings {
    outbound
        .stream_settings
        .get_or_insert_with(|| StreamSettings {
            network: "tcp".to_string(),
            security: None,
            tls_settings: None,
            reality_settings: None,
            ws_settings: None,
            tcp_settings: None,
            grpc_settings: None,
            xhttp_settings: None,
            sockopt: None,
        })
}

fn apply_sockopt(
    stream: &mut StreamSettings,
    interface: Option<String>,
    mark: Option<i64>,
    dialer_proxy: Option<String>,
) {
    let sockopt = stream.sockopt.get_or_insert(Sockopt {
        interface: None,
        mark: None,
        dialer_proxy: None,
    });
    if interface.is_some() {
        sockopt.interface = interface;
    }
    if mark.is_some() {
        sockopt.mark = mark;
    }
    if dialer_proxy.is_some() {
        sockopt.dialer_proxy = dialer_proxy;
    }
}
