//! Bridges runtime tuning and routing app-config sections to generated engine
//! options, plus inbound `listen_interface` resolution.

use crate::app::AppError;
use crate::app::config::{RouteList, RoutingSettings, RuntimeSettings};
use crate::singbox::{SingboxRouteList, SingboxRoutingOptions};
use crate::xray::{FragmentOptions, MuxOptions, XrayGenOptions, XrayRouteList, XrayRoutingOptions};

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
