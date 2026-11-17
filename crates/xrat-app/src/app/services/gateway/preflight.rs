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
        checks.push(conflicts::inspect_tun(tun_name, runtime_dir));
        checks.push(conflicts::inspect_sysctls(
            settings.interface.value.as_deref(),
        ));
        checks.push(check("recovery", "unknown", "Gateway ownership journal and boot-time guard restoration are not implemented; interrupted operation and reboot recovery cannot be verified (#301, #302)."));
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
    checks.push(check("remaining_checks", "unknown", "Firewall, full routing-policy evaluation, marks, ownership, bootstrap, persistence and gateway-specific engine compatibility remain unverified; installation remains unavailable."));
    Report {
        interfaces,
        default_routes,
        policy_rules,
        routes,
        checks,
        ready: false,
    }
}

async fn inspect_runtime_owner(runtime_dir: Option<&std::path::Path>) -> Check {
    #[cfg(target_os = "linux")]
    {
        let Some(runtime_dir) = runtime_dir else {
            return check(
                "runtime_owner",
                "unknown",
                "Runtime directory could not be determined; daemon privileges are unknown.",
            );
        };
        let socket = crate::app::daemon::ipc::default_socket_path(runtime_dir);
        let stream = match tokio::time::timeout(
            std::time::Duration::from_secs(1),
            tokio::net::UnixStream::connect(&socket),
        )
        .await
        {
            Ok(Ok(stream)) => stream,
            Ok(Err(error)) => {
                return check(
                    "runtime_owner",
                    "unknown",
                    format!(
                        "Cannot connect to daemon socket {}: {error}; the gateway runtime owner is unknown.",
                        socket.display()
                    ),
                );
            }
            Err(_) => {
                return check(
                    "runtime_owner",
                    "unknown",
                    "Timed out connecting to the daemon socket; runtime owner privileges are unknown.",
                );
            }
        };
        let Some(pid) = stream
            .peer_cred()
            .ok()
            .and_then(|cred| cred.pid())
            .and_then(|pid| u32::try_from(pid).ok())
        else {
            return check(
                "runtime_owner",
                "unknown",
                "Cannot read daemon socket peer credentials; runtime owner privileges are unknown.",
            );
        };
        match std::fs::read_to_string(format!("/proc/{pid}/status")) {
            Ok(status) => {
                let observed = process_privileges(&status, "This socket peer");
                check(
                    "runtime_owner",
                    observed.state,
                    format!(
                        "Socket peer PID {pid}: {} Peer identity and engine capability inheritance are not verified.",
                        observed.detail
                    ),
                )
            }
            Err(error) => check(
                "runtime_owner",
                "unknown",
                format!("Cannot inspect daemon socket peer PID {pid}: {error}"),
            ),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = runtime_dir;
        check(
            "runtime_owner",
            "unknown",
            "Gateway runtime owner inspection requires Linux.",
        )
    }
}

fn inspect_tun_metadata(metadata: &std::fs::Metadata) -> Check {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        if !metadata.file_type().is_char_device() {
            return check(
                "tun_device",
                "failed",
                "/dev/net/tun exists but is not a character device.",
            );
        }
        check(
            "tun_device",
            "incomplete",
            "/dev/net/tun is a character device; runtime-owner access and TUN creation are not verified.",
        )
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        check(
            "tun_device",
            "unknown",
            "TUN device type cannot be inspected on this platform.",
        )
    }
}

fn inspect_upstream(
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

fn read_routes() -> Result<Vec<RouteObservation>, String> {
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

fn parse_routes(data: &[u8]) -> Result<Vec<RouteObservation>, String> {
    serde_json::from_slice(data)
        .map_err(|error| format!("Cannot parse iproute2 route data: {error}"))
}

fn caller_privileges(status: &str) -> Check {
    process_privileges(status, "This CLI process")
}

fn process_privileges(status: &str, subject: &str) -> Check {
    let field = |name: &str| {
        status
            .lines()
            .find_map(|line| line.strip_prefix(name))
            .map(str::trim)
    };
    let capability = field("CapEff:").and_then(|raw| u64::from_str_radix(raw, 16).ok());
    let no_new_privs = field("NoNewPrivs:").and_then(|raw| match raw {
        "0" => Some(false),
        "1" => Some(true),
        _ => None,
    });
    match (capability, no_new_privs) {
        (Some(mask), Some(no_new_privs)) => check(
            "caller_privileges",
            "incomplete",
            format!(
                "{subject} {} effective CAP_NET_ADMIN; NoNewPrivs={}. Capability inheritance is not verified.",
                if mask & (1 << 12) != 0 {
                    "has"
                } else {
                    "does not have"
                },
                u8::from(no_new_privs)
            ),
        ),
        _ => check(
            "caller_privileges",
            "unknown",
            "Cannot parse effective capabilities and NoNewPrivs for the CLI process.",
        ),
    }
}

fn inspect_engine(engine: &str, path: Option<&std::path::Path>) -> Check {
    if !matches!(engine, "xray" | "sing-box") {
        return check(
            "engine",
            "failed",
            format!(
                "Configured engine '{engine}' is unsupported for the gateway MVP; select Xray or sing-box."
            ),
        );
    }
    let Some(path) = path else {
        return check(
            "engine",
            "unknown",
            format!(
                "Configured engine preference is {engine}; binary path and selected proxy backend are unavailable."
            ),
        );
    };
    let Some(path) = crate::app::tun_privileges::resolve_executable(path) else {
        return check(
            "engine",
            "incomplete",
            format!(
                "Configured {engine} binary was not found at {}; the selected proxy backend is not known.",
                path.display()
            ),
        );
    };
    let version = match engine {
        "sing-box" => xrat_engines::singbox::ensure_supported_binary(&path)
            .map(|version| version.to_string())
            .map_err(|error| error.to_string()),
        _ => {
            let spawner = std::sync::Arc::new(xrat_support::process::SystemProcessSpawner);
            let version = crate::app::services::runtime_tuning::xray_binary_version_with_spawner(
                &path, spawner,
            );
            match version {
                Some((major, minor, patch))
                    if (major, minor, patch)
                        >= crate::app::services::runtime_tuning::XRAY_TUN_MIN_VERSION =>
                {
                    Ok(format!("{major}.{minor}.{patch}"))
                }
                Some((major, minor, patch)) => {
                    let (min_major, min_minor, min_patch) =
                        crate::app::services::runtime_tuning::XRAY_TUN_MIN_VERSION;
                    Err(format!(
                        "Xray {major}.{minor}.{patch} predates the host TUN minimum {min_major}.{min_minor}.{min_patch}"
                    ))
                }
                None => Err("could not determine Xray version".into()),
            }
        }
    };
    match version {
        Ok(version) => check(
            "engine",
            "incomplete",
            format!(
                "Configured {engine} binary {} reports {version} and meets the existing host TUN version gate. The selected proxy backend, gateway configuration and packet behavior are unverified.",
                path.display()
            ),
        ),
        Err(error) => check(
            "engine",
            "incomplete",
            format!(
                "Configured {engine} binary {} did not pass the host TUN version check: {error}. The selected proxy backend is not known.",
                path.display()
            ),
        ),
    }
}

fn select_lan(
    settings: &mut EffectiveSettings,
    addresses: &[InterfaceAddress],
    checks: &mut Vec<Check>,
) {
    let mut candidates: Vec<&str> = addresses
        .iter()
        .filter(|a| a.candidate)
        .map(|a| a.name.as_str())
        .collect();
    candidates.sort_unstable();
    candidates.dedup();
    if settings.interface.value.is_none() && candidates.len() == 1 {
        settings.interface.value = Some(candidates[0].to_string());
        settings.interface.source = "discovery";
    }
    let Some(name) = settings.interface.value.as_deref() else {
        checks.push(check("lan_interface", "failed", format!("Cannot choose an unambiguous LAN interface; candidates: {}. Specify --interface or [gateway].interface.", if candidates.is_empty() { "none".into() } else { candidates.join(", ") })));
        return;
    };
    let mut subnets: Vec<&str> = addresses
        .iter()
        .filter(|a| a.name == name && a.candidate)
        .map(|a| a.subnet.as_str())
        .collect();
    subnets.sort_unstable();
    subnets.dedup();
    if subnets.is_empty() {
        checks.push(check(
            "lan_interface",
            "failed",
            format!("Interface {name} has no eligible active LAN IPv4 address."),
        ));
        return;
    }
    checks.push(check(
        "lan_interface",
        "incomplete",
        format!("Selected {name}; physical LAN reachability has not been verified."),
    ));
    if settings.lan_subnet.value.is_none() && subnets.len() == 1 {
        settings.lan_subnet.value = Some(subnets[0].to_string());
        settings.lan_subnet.source = "discovery";
    }
    match settings.lan_subnet.value.as_deref() {
        Some(subnet) if subnets.contains(&subnet) => checks.push(check(
            "lan_subnet",
            "incomplete",
            format!("{subnet} matches an interface address; client selection is not verified."),
        )),
        Some(subnet) => checks.push(check(
            "lan_subnet",
            "failed",
            format!("{subnet} does not match an eligible subnet on {name}."),
        )),
        None => checks.push(check(
            "lan_subnet",
            "failed",
            format!(
                "Multiple LAN subnets on {name}: {}. Specify --lan-subnet.",
                subnets.join(", ")
            ),
        )),
    }
}

fn parse_default_routes(input: &str) -> Result<Vec<DefaultRoute>, String> {
    let mut routes = Vec::new();
    for line in input.lines().skip(1).filter(|line| !line.trim().is_empty()) {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() < 8 {
            return Err("Malformed /proc/net/route; cannot establish upstream routes.".into());
        }
        let flags = u32::from_str_radix(fields[3], 16).map_err(|_| "Malformed route flags")?;
        if fields[1] != "00000000" || fields[7] != "00000000" || flags & 1 == 0 {
            continue;
        }
        let gateway = u32::from_str_radix(fields[2], 16).map_err(|_| "Malformed route gateway")?;
        let metric = fields[6].parse().map_err(|_| "Malformed route metric")?;
        routes.push(DefaultRoute {
            interface: fields[0].to_string(),
            gateway: Ipv4Addr::from(gateway.to_le_bytes()),
            metric,
        });
    }
    Ok(routes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{
        config::GatewaySettings,
        services::gateway::{GatewayOverrides, resolve},
    };

    fn address(name: &str, subnet: &str) -> InterfaceAddress {
        InterfaceAddress {
            name: name.into(),
            address: Ipv4Addr::new(192, 168, 1, 2),
            subnet: subnet.into(),
            up: true,
            point_to_point: false,
            candidate: true,
        }
    }

    #[test]
    fn chooses_only_unambiguous_interface_and_subnet() {
        let mut settings =
            resolve(&GatewaySettings::default(), &GatewayOverrides::default()).unwrap();
        let mut checks = Vec::new();
        select_lan(
            &mut settings,
            &[address("eth0", "192.168.1.0/24")],
            &mut checks,
        );
        assert_eq!(settings.interface.value.as_deref(), Some("eth0"));
        assert_eq!(settings.lan_subnet.value.as_deref(), Some("192.168.1.0/24"));
        assert_eq!(settings.interface.source, "discovery");
        let mut ambiguous =
            resolve(&GatewaySettings::default(), &GatewayOverrides::default()).unwrap();
        let mut checks = Vec::new();
        select_lan(
            &mut ambiguous,
            &[
                address("eth0", "192.168.1.0/24"),
                address("wlan0", "10.0.0.0/24"),
            ],
            &mut checks,
        );
        assert!(ambiguous.interface.value.is_none());
        assert_eq!(checks[0].state, "failed");
    }

    #[test]
    fn rejects_subnet_mismatch_and_ambiguous_subnets() {
        let mut settings = resolve(
            &GatewaySettings::default(),
            &GatewayOverrides {
                interface: Some("eth0".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let mut checks = Vec::new();
        select_lan(
            &mut settings,
            &[
                address("eth0", "192.168.1.0/24"),
                address("eth0", "10.0.0.0/24"),
            ],
            &mut checks,
        );
        assert!(settings.lan_subnet.value.is_none());
        assert_eq!(checks.last().unwrap().state, "failed");
        settings.lan_subnet.value = Some("172.16.0.0/16".into());
        let mut checks = Vec::new();
        select_lan(
            &mut settings,
            &[address("eth0", "192.168.1.0/24")],
            &mut checks,
        );
        assert_eq!(checks.last().unwrap().state, "failed");
    }

    #[test]
    fn parses_main_table_default_routes_and_rejects_bad_data() {
        let input = "Iface Destination Gateway Flags RefCnt Use Metric Mask\neth0 00000000 0101A8C0 0003 0 0 100 00000000\n";
        let routes = parse_default_routes(input).unwrap();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].gateway, Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(routes[0].metric, 100);
        assert!(parse_default_routes("header\ninvalid route\n").is_err());
    }

    #[test]
    fn upstream_topology_handles_same_interface_and_ambiguous_defaults() {
        let route = |name: &str| DefaultRoute {
            interface: name.into(),
            gateway: Ipv4Addr::new(192, 168, 1, 1),
            metric: 100,
        };
        let shared = inspect_upstream(
            &[route("eth0")],
            &[address("eth0", "192.168.1.0/24")],
            Some("eth0"),
        );
        assert_eq!(shared.state, "incomplete");
        assert!(shared.detail.contains("shares the LAN interface"));
        let separate = inspect_upstream(
            &[route("eth1")],
            &[address("eth1", "192.168.1.0/24")],
            Some("eth0"),
        );
        assert!(separate.detail.contains("separate upstream"));
        let ambiguous = inspect_upstream(
            &[route("eth0"), route("eth1")],
            &[address("eth0", "192.168.1.0/24")],
            Some("eth0"),
        );
        assert_eq!(ambiguous.state, "unknown");
        let missing = inspect_upstream(
            &[route("eth1")],
            &[address("eth0", "192.168.1.0/24")],
            Some("eth0"),
        );
        assert_eq!(missing.state, "failed");
    }

    #[test]
    fn parses_iproute2_tables_and_rejects_invalid_output() {
        let routes = parse_routes(
            br#"[{"dst":"default","gateway":"192.168.1.1","dev":"eth0","table":100,"metric":10}]"#,
        )
        .unwrap();
        assert_eq!(routes[0].table, Some(serde_json::json!(100)));
        assert_eq!(routes[0].gateway.as_deref(), Some("192.168.1.1"));
        assert!(parse_routes(b"invalid").is_err());
    }

    #[test]
    fn caller_capabilities_distinguish_no_new_privileges_from_effective_capability() {
        let check = caller_privileges("CapEff:\t0000000000001000\nNoNewPrivs:\t1\n");
        assert_eq!(check.state, "incomplete");
        assert!(check.detail.contains("has effective CAP_NET_ADMIN"));
        assert!(check.detail.contains("NoNewPrivs=1"));
        assert_eq!(caller_privileges("CapEff: invalid\n").state, "unknown");
        assert!(
            caller_privileges("CapEff: 0\nNoNewPrivs: 0\n")
                .detail
                .contains("does not have")
        );
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn runtime_owner_observes_socket_peer_without_claiming_identity() {
        let dir = tempfile::tempdir().unwrap();
        let socket = crate::app::daemon::ipc::default_socket_path(dir.path());
        let _listener = tokio::net::UnixListener::bind(socket).unwrap();
        let observed = inspect_runtime_owner(Some(dir.path())).await;
        assert_eq!(observed.state, "incomplete");
        assert!(
            observed
                .detail
                .contains(&format!("PID {}", std::process::id()))
        );
        assert!(
            observed
                .detail
                .contains("identity and engine capability inheritance are not verified")
        );
        let missing = tempfile::tempdir().unwrap();
        assert_eq!(
            inspect_runtime_owner(Some(missing.path())).await.state,
            "unknown"
        );
    }

    #[test]
    fn backend_preference_does_not_claim_gateway_support() {
        assert_eq!(inspect_engine("v2ray", None).state, "failed");
        assert_eq!(inspect_engine("xray", None).state, "unknown");
        let missing = inspect_engine(
            "sing-box",
            Some(std::path::Path::new("/xrat-test-missing-engine")),
        );
        assert_eq!(missing.state, "incomplete");
        assert!(
            missing
                .detail
                .contains("selected proxy backend is not known")
        );
    }

    #[cfg(unix)]
    #[test]
    fn xray_version_gate_never_claims_gateway_readiness() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("xray");
        std::fs::write(&binary, "#!/bin/sh\nprintf 'Xray 26.7.11\\n'\n").unwrap();
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755)).unwrap();
        let report = inspect_engine("xray", Some(&binary));
        assert_eq!(report.state, "incomplete");
        assert!(report.detail.contains("26.7.11"));
        assert!(
            report
                .detail
                .contains("gateway configuration and packet behavior are unverified")
        );
        std::fs::write(&binary, "#!/bin/sh\nprintf 'Xray 26.3.27\\n'\n").unwrap();
        let old = inspect_engine("xray", Some(&binary));
        assert_eq!(old.state, "incomplete");
        assert!(old.detail.contains("did not pass"));
    }

    #[cfg(unix)]
    #[test]
    fn singbox_version_gate_is_independent_and_incomplete() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("sing-box");
        std::fs::write(&binary, "#!/bin/sh\nprintf '1.13.21\\n'\n").unwrap();
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755)).unwrap();
        let report = inspect_engine("sing-box", Some(&binary));
        assert_eq!(report.state, "incomplete");
        assert!(report.detail.contains("1.13.21"));
        assert!(report.detail.contains("packet behavior are unverified"));
        std::fs::write(&binary, "#!/bin/sh\nprintf '1.12.0\\n'\n").unwrap();
        let old = inspect_engine("sing-box", Some(&binary));
        assert_eq!(old.state, "incomplete");
        assert!(old.detail.contains("did not pass"));
    }
}
