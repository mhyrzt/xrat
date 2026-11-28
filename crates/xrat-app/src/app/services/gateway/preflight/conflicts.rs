use super::{Check, check};
use xrat_support::net::KernelPolicyRule;

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

#[cfg(test)]
mod tests;
