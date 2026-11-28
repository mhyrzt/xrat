use super::*;

pub(super) fn inspect_upstream(
    routes: &[DefaultRoute],
    interfaces: &[InterfaceAddress],
    lan: Option<&str>,
) -> Check {
    if routes.is_empty() {
        return check(
            "upstream_topology",
            "unknown",
            "No main-table default route was observed; policy-based or unavailable upstream paths require further inspection.",
        );
    }
    let Some(metric) = routes.iter().map(|route| route.metric).min() else {
        return check(
            "upstream_topology",
            "unknown",
            "No usable main-table default route was observed.",
        );
    };
    let best: Vec<_> = routes
        .iter()
        .filter(|route| route.metric == metric)
        .collect();
    if best.len() != 1 {
        return check(
            "upstream_topology",
            "unknown",
            format!(
                "{} main-table default routes have the same lowest metric {metric}; select or resolve the upstream before installation.",
                best.len()
            ),
        );
    }
    let upstream = best[0];
    if interfaces.is_empty() {
        return check(
            "upstream_topology",
            "unknown",
            format!(
                "Main-table default uses {}, but interface addresses could not be inspected.",
                upstream.interface
            ),
        );
    }
    if !interfaces
        .iter()
        .any(|iface| iface.name == upstream.interface && iface.up)
    {
        return check(
            "upstream_topology",
            "failed",
            format!(
                "Main-table default uses {}, which has no observed active IPv4 interface; verify the upstream route.",
                upstream.interface
            ),
        );
    }
    if upstream.gateway.is_unspecified() {
        return check(
            "upstream_topology",
            "unknown",
            format!(
                "Main-table default through {} has no gateway; its physical next hop requires explicit verification.",
                upstream.interface
            ),
        );
    }
    let relationship = match lan {
        Some(interface) if interface == upstream.interface => "shares the LAN interface",
        Some(_) => "uses a separate upstream interface",
        None => "cannot be compared with an undetermined LAN interface",
    };
    check(
        "upstream_topology",
        "incomplete",
        format!(
            "Main-table default via {} on {} {relationship}; RPDB selection, next-hop reachability and proxy bootstrap are not verified.",
            upstream.gateway, upstream.interface
        ),
    )
}

pub(super) fn read_routes() -> Result<Vec<RouteObservation>, String> {
    let output = xrat_support::process::Command::new("ip")
        .args(["-j", "-4", "route", "show", "table", "all"])
        .output()
        .map_err(|error| format!("Cannot run read-only iproute2 route inspection: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Cannot inspect IPv4 routing tables: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    parse_routes(&output.stdout)
}

pub(super) fn parse_routes(data: &[u8]) -> Result<Vec<RouteObservation>, String> {
    serde_json::from_slice(data)
        .map_err(|error| format!("Cannot parse iproute2 route data: {error}"))
}
