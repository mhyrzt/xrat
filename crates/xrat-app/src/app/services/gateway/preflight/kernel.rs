use super::{Check, check};
use std::path::Path;
use xrat_support::net::{SystemTunInterfaceOps, TunInterfaceOps};

pub(super) fn inspect_tun(name: &str, runtime_dir: Option<&Path>) -> Check {
    if name.is_empty()
        || name.len() > 15
        || name == "."
        || name == ".."
        || name
            .bytes()
            .any(|byte| byte == b'/' || byte.is_ascii_whitespace() || byte.is_ascii_control())
    {
        return check(
            "tun_collision",
            "unknown",
            "Configured TUN interface name is invalid; ownership cannot be inspected.",
        );
    }
    let interface = match SystemTunInterfaceOps.inspect_interface(name) {
        Ok(interface) => interface,
        Err(error) => {
            return check(
                "tun_collision",
                "unknown",
                format!("Cannot inspect configured TUN interface {name}: {error}"),
            );
        }
    };
    let Some(runtime_dir) = runtime_dir else {
        return check(
            "tun_collision",
            "unknown",
            "Runtime directory is unavailable; TUN ownership cannot be inspected.",
        );
    };
    let journal = crate::app::runtime_service::tun_ownership::ownership_path(runtime_dir);
    let record = match std::fs::read(&journal) {
        Ok(data) => match serde_json::from_slice::<
            crate::app::runtime_service::tun_ownership::TunOwnershipRecord,
        >(&data)
        {
            Ok(record) => Some(record),
            Err(error) => {
                return check(
                    "tun_collision",
                    "unknown",
                    format!(
                        "Host TUN ownership record {} is invalid: {error}; do not replace or remove resources.",
                        journal.display()
                    ),
                );
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return check(
                "tun_collision",
                "unknown",
                format!(
                    "Cannot inspect host TUN ownership record {}: {error}",
                    journal.display()
                ),
            );
        }
    };
    match (interface, record) {
        (Some(interface), Some(record))
            if record.interface_name == name
                && record.ifindex == Some(interface.ifindex)
                && interface.is_tun =>
        {
            check(
                "tun_collision",
                "incomplete",
                format!(
                    "Interface {name} matches the host TUN record's index; live ownership and gateway coexistence require verification."
                ),
            )
        }
        (Some(interface), _) => check(
            "tun_collision",
            "unknown",
            format!(
                "Interface {name} (index {}) exists without matching host TUN ownership; do not replace or remove it.",
                interface.ifindex
            ),
        ),
        (None, Some(record)) if record.interface_name == name => check(
            "tun_collision",
            "unknown",
            format!(
                "Host TUN record references absent interface {name}; reconcile ownership before gateway changes."
            ),
        ),
        (None, _) => check(
            "tun_collision",
            "incomplete",
            format!(
                "No interface named {name} was observed; gateway ownership and collisions remain unverified."
            ),
        ),
    }
}

pub(super) fn inspect_sysctls(interface: Option<&str>) -> Check {
    let mut paths = vec![
        "net/ipv4/ip_forward".to_string(),
        "net/ipv4/conf/all/rp_filter".to_string(),
        "net/ipv4/conf/all/src_valid_mark".to_string(),
    ];
    if let Some(name) = interface {
        if name.is_empty() || name.contains('/') || name == "." || name == ".." {
            return check(
                "network_sysctls",
                "unknown",
                "Invalid LAN interface name; cannot inspect interface-specific IPv4 settings.",
            );
        }
        paths.extend([
            format!("net/ipv4/conf/{name}/rp_filter"),
            format!("net/ipv4/conf/{name}/src_valid_mark"),
        ]);
    }
    let mut values = Vec::new();
    for name in paths {
        match std::fs::read_to_string(Path::new("/proc/sys").join(&name)) {
            Ok(value) => match value.trim().parse::<u32>() {
                Ok(value) => values.push(format!("{name}={value}")),
                Err(_) => {
                    return check(
                        "network_sysctls",
                        "unknown",
                        format!("Cannot parse /proc/sys/{name}"),
                    );
                }
            },
            Err(error) => {
                return check(
                    "network_sysctls",
                    "unknown",
                    format!("Cannot read /proc/sys/{name}: {error}"),
                );
            }
        }
    }
    check(
        "network_sysctls",
        "incomplete",
        format!(
            "{}; routing compatibility and safe restoration are not verified.",
            values.join(", ")
        ),
    )
}
