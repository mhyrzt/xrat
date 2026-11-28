use std::path::Path;

use super::{Check, check};
use xrat_support::net::{KernelPolicyRule, SystemTunInterfaceOps, TunInterfaceOps};

pub(super) fn inspect_rule_marks(rules: Option<&[KernelPolicyRule]>) -> Check {
    let Some(rules) = rules else {
        return check(
            "rule_marks",
            "unknown",
            "Policy rules could not be inspected; mark and priority conflicts are unknown.",
        );
    };
    let marked: Vec<_> = rules
        .iter()
        .filter(|rule| {
            rule.attributes
                .iter()
                .any(|attribute| matches!(attribute.kind, 10 | 16))
        })
        .collect();
    let priorities = marked
        .iter()
        .map(|rule| {
            rule.priority()
                .map_or_else(|| "unknown".into(), |priority| priority.to_string())
        })
        .collect::<Vec<_>>()
        .join(", ");
    check(
        "rule_marks",
        "incomplete",
        format!(
            "Observed {} marked IPv4 policy rules{}; gateway marks, masks and ownership are not yet defined, so conflicts cannot be ruled out.",
            marked.len(),
            if priorities.is_empty() {
                String::new()
            } else {
                format!(" at priorities {priorities}")
            }
        ),
    )
}

#[derive(serde::Deserialize)]
struct NftOutput {
    nftables: Vec<serde_json::Value>,
}

fn inspect_nft_output(data: &[u8]) -> Result<Check, String> {
    let output: NftOutput = serde_json::from_slice(data)
        .map_err(|error| format!("Cannot parse nftables ruleset: {error}"))?;
    let mut hooks = Vec::new();
    let mut host_guard = false;
    for entry in output.nftables {
        if entry.get("flowtable").is_some() {
            return Ok(check(
                "firewall_chains",
                "failed",
                "nftables flowtable offload can bypass the gateway forwarding chain.",
            ));
        }
        if let Some(table) = entry.get("table") {
            host_guard |= table.get("family").and_then(|value| value.as_str()) == Some("inet")
                && table.get("name").and_then(|value| value.as_str())
                    == Some(xrat_support::net::firewall::nftables::TABLE_NAME);
        }
        if let Some(chain) = entry.get("chain") {
            let Some(hook) = chain.get("hook").and_then(|value| value.as_str()) else {
                continue;
            };
            let family = chain
                .get("family")
                .and_then(|value| value.as_str())
                .ok_or("nftables base chain has no family")?;
            let table = chain
                .get("table")
                .and_then(|value| value.as_str())
                .ok_or("nftables base chain has no table")?;
            let name = chain
                .get("name")
                .and_then(|value| value.as_str())
                .ok_or("nftables base chain has no name")?;
            let priority = chain
                .get("prio")
                .or_else(|| chain.get("priority"))
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".into());
            hooks.push(format!(
                "{family}/{table}/{name} hook {hook} priority {priority}"
            ));
        }
    }
    let summary = if hooks.is_empty() {
        "No base chains observed".to_string()
    } else {
        format!("Base chains: {}", hooks.join("; "))
    };
    Ok(check(
        "firewall_chains",
        "incomplete",
        format!(
            "{summary}. Host OUTPUT kill-switch table {}; chain ownership, iptables compatibility and gateway hook conflicts are unverified.",
            if host_guard {
                "is present; it does not protect forwarded traffic"
            } else {
                "was not observed"
            }
        ),
    ))
}

pub(super) fn inspect_firewall() -> Check {
    match xrat_support::process::Command::new("nft")
        .args(["-j", "list", "ruleset"])
        .output()
    {
        Ok(output) if output.status.success() => inspect_nft_output(&output.stdout)
            .unwrap_or_else(|error| check("firewall_chains", "unknown", error)),
        Ok(output) => check(
            "firewall_chains",
            "unknown",
            format!(
                "Cannot inspect nftables ruleset ({}): {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ),
        Err(error) => check(
            "firewall_chains",
            "unknown",
            format!("Cannot run read-only nftables inspection: {error}"),
        ),
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use xrat_support::net::KernelRuleAttribute;

    #[test]
    fn nftables_flowtable_blocks_gateway_preflight() {
        let report = inspect_nft_output(
            br#"{"nftables":[{"flowtable":{"family":"inet","table":"foreign","name":"fastpath"}}]}"#,
        ).unwrap();
        assert_eq!(report.state, "failed");
    }

    #[test]
    fn nftables_reports_base_chain_and_host_guard_without_asserting_protection() {
        let data = br#"{"nftables":[{"table":{"family":"inet","name":"xrat_killswitch"}},{"chain":{"family":"inet","table":"foreign","name":"forward","hook":"forward","prio":0}}]}"#;
        let result = inspect_nft_output(data).unwrap();
        assert_eq!(result.state, "incomplete");
        assert!(
            result
                .detail
                .contains("inet/foreign/forward hook forward priority 0")
        );
        assert!(result.detail.contains("does not protect forwarded traffic"));
        assert!(inspect_nft_output(br#"{"nftables":[{"chain":{"hook":"forward"}}]}"#).is_err());
        assert!(inspect_nft_output(b"invalid").is_err());
    }

    #[test]
    fn marks_unknown_when_rules_unavailable_and_do_not_claim_conflict_free() {
        assert_eq!(inspect_rule_marks(None).state, "unknown");
        let rule = KernelPolicyRule {
            family: 2,
            destination_prefix: 0,
            source_prefix: 0,
            tos: 0,
            table: 100,
            action: 1,
            flags: 0,
            attributes: vec![
                KernelRuleAttribute {
                    kind: 10,
                    value: 1u32.to_ne_bytes().to_vec(),
                },
                KernelRuleAttribute {
                    kind: 6,
                    value: 100u32.to_ne_bytes().to_vec(),
                },
            ],
        };
        let result = inspect_rule_marks(Some(&[rule]));
        assert_eq!(result.state, "incomplete");
        assert!(result.detail.contains("priorities 100"));
        assert!(result.detail.contains("conflicts cannot be ruled out"));
    }
}
