mod intent;
mod kernel;

mod engine;
mod privileges;
use engine::{inspect_engine, inspect_tun_metadata};
use privileges::{caller_privileges, inspect_runtime_owner};

mod routing;
mod topology;
use routing::{inspect_upstream, read_routes};
use topology::{parse_default_routes, select_lan};
#[cfg(test)]
mod tests;

use std::net::Ipv4Addr;

use serde::Serialize;

use super::EffectiveSettings;
use xrat_support::net::TunInterfaceOps;
mod conflicts;

#[derive(Debug, Serialize)]
pub struct InterfaceAddress {
    pub(crate) name: String,
    pub(crate) address: Ipv4Addr,
    pub(crate) subnet: String,
    pub(crate) up: bool,
    pub(crate) point_to_point: bool,
    pub(crate) candidate: bool,
}

#[derive(Debug, Serialize)]
pub struct DefaultRoute {
    pub(crate) interface: String,
    pub(crate) gateway: Ipv4Addr,
    pub(crate) metric: u32,
}

#[derive(Debug, Serialize)]
pub struct Check {
    pub(crate) name: &'static str,
    pub(crate) state: &'static str,
    pub(crate) detail: String,
}

#[derive(Debug, Serialize, serde::Deserialize)]
struct RouteObservation {
    #[serde(default)]
    dst: Option<String>,
    #[serde(default)]
    gateway: Option<String>,
    #[serde(default)]
    dev: Option<String>,
    #[serde(default)]
    table: Option<serde_json::Value>,
    #[serde(default)]
    metric: Option<u32>,
    #[serde(default, rename = "type")]
    route_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub(crate) interfaces: Vec<InterfaceAddress>,
    pub(crate) default_routes: Vec<DefaultRoute>,
    pub(crate) policy_rules: Vec<xrat_support::net::KernelPolicyRule>,
    routes: Vec<RouteObservation>,
    pub(crate) checks: Vec<Check>,
    ready: bool,
}

fn check(name: &'static str, state: &'static str, detail: impl Into<String>) -> Check {
    Check {
        name,
        state,
        detail: detail.into(),
    }
}

pub async fn inspect(
    settings: &mut EffectiveSettings,
    engine: &str,
    engine_path: Option<&std::path::Path>,
    runtime_dir: Option<&std::path::Path>,
    tun_name: &str,
) -> Report {
    let mut checks = Vec::new();
    let mut interfaces = Vec::new();
    let mut default_routes = Vec::new();
    let mut policy_rules = Vec::new();
    let mut policy_rules_observed = false;
    let mut routes = Vec::new();
    if !cfg!(target_os = "linux") {
        checks.push(check("platform", "failed", "Gateway mode requires Linux."));
    } else {
        checks.push(check(
            "platform",
            "ok",
            "Linux detected; architecture and backend support remain unverified.",
        ));
        match if_addrs::get_if_addrs() {
            Ok(addresses) => {
                for iface in addresses {
                    let if_addrs::IfAddr::V4(addr) = &iface.addr else {
                        continue;
                    };
                    if addr.prefixlen > 32 {
                        continue;
                    }
                    let mask = if addr.prefixlen == 0 {
                        0
                    } else {
                        u32::MAX << (32 - addr.prefixlen)
                    };
                    let subnet = format!(
                        "{}/{}",
                        Ipv4Addr::from(u32::from(addr.ip) & mask),
                        addr.prefixlen
                    );
                    interfaces.push(InterfaceAddress {
                        name: iface.name.clone(),
                        address: addr.ip,
                        subnet,
                        up: iface.is_oper_up(),
                        point_to_point: iface.is_p2p(),
                        candidate: iface.is_oper_up()
                            && !iface.is_loopback()
                            && !iface.is_link_local()
                            && !iface.is_p2p()
                            && addr.prefixlen > 0
                            && addr.prefixlen < 31,
                    });
                }
                interfaces.sort_by(|a, b| (&a.name, a.address).cmp(&(&b.name, b.address)));
                select_lan(settings, &interfaces, &mut checks);
            }
            Err(error) => checks.push(check(
                "interfaces",
                "unknown",
                format!("Cannot inspect interfaces: {error}"),
            )),
        }
        match std::fs::read_to_string("/proc/net/route") {
            Ok(routes) => match parse_default_routes(&routes) {
                Ok(found) => {
                    default_routes = found;
                    let detail = if default_routes.is_empty() {
                        "No active IPv4 default route found in the main table; policy rules and other tables are not verified."
                    } else {
                        "Main-table IPv4 defaults found; policy rules, other tables, multipath and physical upstream identity are not verified."
                    };
                    checks.push(check(
                        "upstream_routes",
                        if default_routes.is_empty() {
                            "failed"
                        } else {
                            "incomplete"
                        },
                        detail,
                    ));
                }
                Err(error) => checks.push(check("upstream_routes", "unknown", error)),
            },
            Err(error) => checks.push(check(
                "upstream_routes",
                "unknown",
                format!("Cannot read main-table routes: {error}"),
            )),
        }
        match xrat_support::net::SystemTunInterfaceOps.policy_rules() {
            Ok(rules) => {
                policy_rules_observed = true;
                policy_rules = rules.into_iter().filter(|rule| rule.family == 2).collect();
                let singbox_rules = policy_rules
                    .iter()
                    .filter(|rule| rule.is_singbox_capture_rule())
                    .count();
                checks.push(check("policy_rules", "incomplete", format!("Read {} IPv4 policy rules, including {singbox_rules} in sing-box's capture-priority range; ownership, selectors and routing-table conflicts require further inspection.", policy_rules.len())));
            }
            Err(error) => checks.push(check(
                "policy_rules",
                "unknown",
                format!("Cannot inspect IPv4 policy rules: {error}"),
            )),
        }
        checks.push(conflicts::inspect_rule_marks(
            policy_rules_observed.then_some(policy_rules.as_slice()),
        ));
        checks.push(conflicts::inspect_firewall());
        checks.push(kernel::inspect_tun(tun_name, runtime_dir));
        checks.push(kernel::inspect_sysctls(settings.interface.value.as_deref()));
        checks.push(inspect_ipv6_boundary(settings.interface.value.as_deref()));
        checks.extend(intent::inspect_gateway_intent(runtime_dir));
        checks.push(check("recovery", "unknown", "Gateway boot recovery requires the recorded LAN identity and disabled forwarding until the closed guard is restored; current observations cannot verify a future reboot."));
        checks.push(inspect_upstream(
            &default_routes,
            &interfaces,
            settings.interface.value.as_deref(),
        ));
        match read_routes() {
            Ok(found) => {
                routes = found;
                checks.push(check("routing_tables", "incomplete", format!("Read {} IPv4 route entries from iproute2; policy evaluation, multipath, reachability and ownership are not verified.", routes.len())));
            }
            Err(error) => checks.push(check("routing_tables", "unknown", error)),
        }
        checks.push(match std::fs::read_to_string("/proc/self/status") {
            Ok(status) => caller_privileges(&status),
            Err(error) => check(
                "caller_privileges",
                "unknown",
                format!("Cannot inspect caller privileges: {error}"),
            ),
        });
        checks.push(match std::fs::metadata("/dev/net/tun") {
            Ok(metadata) => inspect_tun_metadata(&metadata),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => check(
                "tun_device",
                "failed",
                "/dev/net/tun was not found in this process's filesystem namespace.",
            ),
            Err(error) => check(
                "tun_device",
                "unknown",
                format!("Cannot inspect /dev/net/tun: {error}"),
            ),
        });
        checks.push(inspect_runtime_owner(runtime_dir).await);
        checks.push(match crate::app::tun_privileges::resolve_executable(std::path::Path::new("nft")) {
            Some(path) => check("firewall_tool", "incomplete", format!("Found nft at {}; executability, kernel support and firewall conflicts are not verified.", path.display())),
            None => check("firewall_tool", "incomplete", "nft was not found on the current PATH; the runtime owner's environment and firewall integration are not verified."),
        });
        checks.push(inspect_engine(engine, engine_path));
    }
    checks.push(check("remaining_checks", "unknown", "Preflight cannot prove end-to-end packet forwarding, proxy bootstrap, or recovery after reboot. Verify these on the intended gateway with the selected engine."));
    Report {
        interfaces,
        default_routes,
        policy_rules,
        routes,
        checks,
        ready: false,
    }
}

fn inspect_ipv6_boundary(interface: Option<&str>) -> Check {
    let observations = [Some("all"), interface]
        .into_iter()
        .flatten()
        .map(|device| {
            let path = format!("/proc/sys/net/ipv6/conf/{device}/forwarding");
            match std::fs::read_to_string(path) {
                Ok(value) => format!("{device}={}", value.trim()),
                Err(_) => format!("{device}=unknown"),
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    check(
        "ipv6_boundary",
        "incomplete",
        format!(
            "IPv6 forwarding ({observations}) is outside gateway protection. Independent client IPv6 routes cannot be verified on the gateway."
        ),
    )
}
