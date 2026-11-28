use std::io;
#[cfg(target_os = "linux")]
use std::{fmt::Write, num::NonZeroU32};

use serde_json::{Value, json};
use xrat_support::net::{SystemTunInterfaceOps, TunInterfaceOps};

use super::journal::InstallIntent;

#[cfg(target_os = "linux")]
use xrat_support::net::firewall::gateway::{
    CAPTURE_COMMENT, CAPTURE_DROP_COMMENT, RETURN_COMMENT, TABLE,
};
#[cfg(not(target_os = "linux"))]
const TABLE: &str = "xrat_gateway";

/// The closed baseline allows host traffic and blocks IPv4 forwarding from the
/// LAN. Activation adds narrowly scoped exceptions to this guard atomically.
#[cfg(target_os = "linux")]
fn closed_rules(intent: &InstallIntent, lan_ifindex: NonZeroU32) -> String {
    let mut rules = String::new();
    writeln!(
        rules,
        "create table ip {TABLE} {{ comment \"xrat gateway {}\"; }}",
        intent.installation_id
    )
    .unwrap();
    writeln!(
        rules,
        "add chain ip {TABLE} forward {{ type filter hook forward priority -10; policy accept; }}"
    )
    .unwrap();
    for subnet in &intent.local_subnets {
        writeln!(
            rules,
            "add rule ip {TABLE} forward meta iif {lan_ifindex} meta oif {lan_ifindex} ip saddr {} ip daddr {subnet} counter accept",
            intent.lan_subnet
        ).unwrap();
        writeln!(
            rules,
            "add rule ip {TABLE} forward meta iif {lan_ifindex} meta oif {lan_ifindex} ip saddr {subnet} ip daddr {} ct state established,related counter accept",
            intent.lan_subnet
        ).unwrap();
    }
    writeln!(
        rules,
        "add rule ip {TABLE} forward meta iif {lan_ifindex} counter drop"
    )
    .unwrap();
    rules
}

#[cfg(target_os = "linux")]
pub(super) fn install_closed(intent: &InstallIntent) -> std::io::Result<()> {
    let index = NonZeroU32::new(intent.lan_ifindex).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid LAN index")
    })?;
    for subnet in std::iter::once(&intent.lan_subnet).chain(&intent.local_subnets) {
        super::validate_subnet(subnet).map_err(io::Error::other)?;
    }
    xrat_support::net::firewall::nftables::apply_transaction(&closed_rules(intent, index))
}

pub(super) fn inspect_closed(intent: &InstallIntent) -> io::Result<bool> {
    if !matches_lan(intent)? {
        return Ok(false);
    }

    let output = xrat_support::process::Command::new("nft")
        .args(["-j", "-n", "list", "tables", "ip"])
        .output()?;
    let tables = read_objects(output)?;
    if !tables.iter().any(|object| object["table"]["name"] == TABLE) {
        return Ok(false);
    }

    let output = xrat_support::process::Command::new("nft")
        .args(["-j", "-n", "list", "table", "ip", TABLE])
        .output()?;
    Ok(matches_closed(&read_objects(output)?, intent))
}

#[cfg(target_os = "linux")]
pub(super) fn verify_no_flow_offload() -> io::Result<()> {
    let output = xrat_support::process::Command::new("nft")
        .args(["-j", "list", "ruleset"])
        .output()?;
    if read_objects(output)?
        .iter()
        .any(|object| object.get("flowtable").is_some())
    {
        return Err(io::Error::other(
            "nftables flowtable offload can bypass the gateway forwarding chain",
        ));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub(super) fn remove_closed(intent: &InstallIntent) -> io::Result<()> {
    if !matches_lan(intent)? {
        return Err(io::Error::other("gateway LAN interface identity changed"));
    }
    let tables = read_objects(
        xrat_support::process::Command::new("nft")
            .args(["-j", "-n", "list", "tables", "ip"])
            .output()?,
    )?;
    if !tables.iter().any(|object| object["table"]["name"] == TABLE) {
        return Ok(());
    }
    let objects = read_objects(
        xrat_support::process::Command::new("nft")
            .args(["-j", "-n", "-a", "list", "table", "ip", TABLE])
            .output()?,
    )?;
    if !matches_closed(&objects, intent) {
        return Err(io::Error::other(
            "gateway guard differs from the expected owned shape",
        ));
    }
    let handle = objects[0]["table"]["handle"]
        .as_u64()
        .ok_or_else(|| io::Error::other("gateway table handle is missing"))?;
    xrat_support::net::firewall::nftables::apply_transaction(&format!(
        "delete table ip handle {handle}\n"
    ))
}

#[cfg(target_os = "linux")]
pub(super) fn close_capture(
    intent: &InstallIntent,
    capture_name: &str,
    capture_index: NonZeroU32,
) -> io::Result<()> {
    if !matches_lan(intent)? {
        return Err(io::Error::other("gateway LAN interface identity changed"));
    }
    let objects = read_objects(
        xrat_support::process::Command::new("nft")
            .args(["-j", "-n", "-a", "list", "table", "ip", TABLE])
            .output()?,
    )?;
    if matches_closed(&objects, intent) {
        return Ok(());
    }
    let handles =
        active_handles(&objects, intent, capture_name, capture_index).ok_or_else(|| {
            io::Error::other("active gateway rules differ from the expected owned shape")
        })?;
    let mut transaction = String::new();
    for handle in handles {
        writeln!(
            transaction,
            "delete rule ip {TABLE} forward handle {handle}"
        )
        .unwrap();
    }
    xrat_support::net::firewall::nftables::apply_transaction(&transaction)?;
    if !inspect_closed(intent)? {
        return Err(io::Error::other(
            "could not verify the closed gateway guard after removing capture rules",
        ));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub(super) fn inspect_active(
    intent: &InstallIntent,
    capture_name: &str,
    capture_index: NonZeroU32,
) -> io::Result<bool> {
    let Some(interface) = SystemTunInterfaceOps.inspect_interface(capture_name)? else {
        return Ok(false);
    };
    if interface.ifindex != capture_index.get() || !matches_lan(intent)? {
        return Ok(false);
    }
    let output = xrat_support::process::Command::new("nft")
        .args(["-j", "-n", "-a", "list", "table", "ip", TABLE])
        .output()?;
    Ok(active_handles(&read_objects(output)?, intent, capture_name, capture_index).is_some())
}

#[cfg(target_os = "linux")]
fn active_handles(
    objects: &[Value],
    intent: &InstallIntent,
    capture_name: &str,
    capture_index: NonZeroU32,
) -> Option<[u64; 3]> {
    let [table, chain, ingress, reply, capture_drop, base @ ..] = objects else {
        return None;
    };
    let base = [table, chain]
        .into_iter()
        .chain(base)
        .cloned()
        .collect::<Vec<_>>();
    if !matches_closed(&base, intent) {
        return None;
    }
    let (network, prefix) = intent.lan_subnet.split_once('/')?;
    let subnet = json!({"prefix": {"addr": network, "len": prefix.parse::<u8>().ok()?}});
    let ingress = &ingress["rule"];
    let reply = &reply["rule"];
    let capture_drop = &capture_drop["rule"];
    for (rule, comment, count, verdict) in [
        (ingress, CAPTURE_COMMENT, 5, "accept"),
        (reply, RETURN_COMMENT, 6, "accept"),
        (capture_drop, CAPTURE_DROP_COMMENT, 3, "drop"),
    ] {
        let expressions = rule["expr"].as_array()?;
        if rule["family"] != "ip"
            || rule["table"] != TABLE
            || rule["chain"] != "forward"
            || rule["comment"] != comment
            || expressions.len() != count
            || !expressions[count - 2]["counter"]["packets"].is_u64()
            || !expressions[count - 2]["counter"]["bytes"].is_u64()
            || expressions[count - 1] != json!({(verdict): null})
        {
            return None;
        }
    }
    let match_interface = |expression: &Value, key, name, index: u32| {
        let value = &expression["match"]["right"];
        (value == name || value == index || value == &index.to_string())
            && *expression
                == json!({"match": {"op": "==", "left": {"meta": {"key": key}}, "right": value}})
    };
    let match_lan = |expression, key| {
        match_interface(
            expression,
            key,
            intent.lan_interface.as_str(),
            intent.lan_ifindex,
        )
    };
    let match_capture =
        |expression, key| match_interface(expression, key, capture_name, capture_index.get());
    let match_subnet = |expression: &Value, field| {
        *expression
            == json!({
                "match": {"op": "==", "left": {"payload": {"protocol": "ip", "field": field}}, "right": subnet}
            })
    };
    let open = &ingress["expr"];
    let return_path = &reply["expr"];
    if !match_lan(&open[0], "iif")
        || !match_capture(&open[1], "oif")
        || !match_subnet(&open[2], "saddr")
        || !match_capture(&return_path[0], "iif")
        || !match_lan(&return_path[1], "oif")
        || !match_subnet(&return_path[2], "daddr")
        || return_path[3]
            != json!({"match": {"op": "in", "left": {"ct": {"key": "state"}}, "right": [2, 4]}})
        || !match_capture(&capture_drop["expr"][0], "iif")
    {
        return None;
    }
    Some([
        ingress["handle"].as_u64()?,
        reply["handle"].as_u64()?,
        capture_drop["handle"].as_u64()?,
    ])
}

pub(super) fn matches_lan(intent: &InstallIntent) -> io::Result<bool> {
    let matching_index = SystemTunInterfaceOps
        .inspect_interface(&intent.lan_interface)?
        .is_some_and(|interface| interface.ifindex == intent.lan_ifindex && !interface.is_tun);
    if !matching_index {
        return Ok(false);
    }
    let addresses = if_addrs::get_if_addrs()?;
    let addresses = addresses
        .iter()
        .filter(|interface| interface.name == intent.lan_interface)
        .filter_map(|interface| match &interface.addr {
            if_addrs::IfAddr::V4(address) => Some(address),
            _ => None,
        })
        .collect::<Vec<_>>();
    Ok(addresses
        .iter()
        .any(|address| address.ip == intent.lan_address)
        && intent.local_subnets.iter().all(|subnet| {
            addresses.iter().any(|address| {
                let network =
                    std::net::Ipv4Addr::from(u32::from(address.ip) & u32::from(address.netmask));
                *subnet == format!("{network}/{}", address.prefixlen)
            })
        }))
}

fn read_objects(output: std::process::Output) -> io::Result<Vec<Value>> {
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "nftables inspection failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    #[derive(serde::Deserialize)]
    struct Ruleset {
        nftables: Vec<Value>,
    }

    let ruleset: Ruleset = serde_json::from_slice(&output.stdout)?;
    Ok(ruleset
        .nftables
        .into_iter()
        .filter(|object| object.get("metainfo").is_none())
        .collect())
}

fn matches_closed(objects: &[Value], intent: &InstallIntent) -> bool {
    let [table, chain, local @ .., rule] = objects else {
        return false;
    };
    if local.len() != intent.local_subnets.len() * 2
        || !intent
            .local_subnets
            .iter()
            .zip(local.chunks_exact(2))
            .all(|(subnet, rules)| {
                matches_local_rule(&rules[0]["rule"], intent, &intent.lan_subnet, subnet, false)
                    && matches_local_rule(
                        &rules[1]["rule"],
                        intent,
                        subnet,
                        &intent.lan_subnet,
                        true,
                    )
            })
    {
        return false;
    }
    let table = &table["table"];
    let chain = &chain["chain"];
    let rule = &rule["rule"];
    let Some(expressions) = rule["expr"].as_array() else {
        return false;
    };
    let [selector, counter, verdict] = expressions.as_slice() else {
        return false;
    };
    let right = &selector["match"]["right"];
    table["family"] == "ip"
        && table["name"] == TABLE
        && table
            .get("flags")
            .is_none_or(|flags| flags.as_array().is_some_and(Vec::is_empty))
        && table["comment"] == format!("xrat gateway {}", intent.installation_id)
        && chain["family"] == "ip"
        && chain["table"] == TABLE
        && chain["name"] == "forward"
        && chain["type"] == "filter"
        && chain["hook"] == "forward"
        && chain["prio"] == -10
        && chain["policy"] == "accept"
        && rule["family"] == "ip"
        && rule["table"] == TABLE
        && rule["chain"] == "forward"
        && (right == &intent.lan_interface || right == intent.lan_ifindex)
        && *selector
            == json!({"match": {"op": "==", "left": {"meta": {"key": "iif"}}, "right": right}})
        && counter["counter"]["packets"].is_u64()
        && counter["counter"]["bytes"].is_u64()
        && *verdict == json!({"drop": null})
}

fn matches_local_rule(
    rule: &Value,
    intent: &InstallIntent,
    source: &str,
    destination: &str,
    reply: bool,
) -> bool {
    let Some(expr) = rule["expr"].as_array() else {
        return false;
    };
    let count = if reply { 7 } else { 6 };
    if expr.len() != count
        || rule["family"] != "ip"
        || rule["table"] != TABLE
        || rule["chain"] != "forward"
    {
        return false;
    }
    let interface = |expression: &Value, key| {
        let right = &expression["match"]["right"];
        (right == &intent.lan_interface || right == intent.lan_ifindex)
            && *expression
                == json!({"match": {"op": "==", "left": {"meta": {"key": key}}, "right": right}})
    };
    let subnet = |expression: &Value, field, cidr: &str| {
        cidr.split_once('/').and_then(|(network, prefix)| Some((network, prefix.parse::<u8>().ok()?)))
            .is_some_and(|(network, prefix)| {
                *expression == json!({"match": {"op": "==", "left": {"payload": {"protocol": "ip", "field": field}}, "right": {"prefix": {"addr": network, "len": prefix}}}})
            })
    };
    interface(&expr[0], "iif")
        && interface(&expr[1], "oif")
        && subnet(&expr[2], "saddr", source)
        && subnet(&expr[3], "daddr", destination)
        && (!reply
            || expr[4]
                == json!({"match": {"op": "in", "left": {"ct": {"key": "state"}}, "right": [2, 4]}}))
        && expr[count - 2]["counter"]["packets"].is_u64()
        && expr[count - 2]["counter"]["bytes"].is_u64()
        && expr[count - 1] == json!({"accept": null})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_altered_guard_even_with_the_same_ownership_marker() {
        let intent = InstallIntent {
            installation_id: uuid::Uuid::new_v4(),
            lan_interface: "eth0".into(),
            lan_ifindex: 2,
            lan_address: std::net::Ipv4Addr::new(192, 0, 2, 2),
            lan_subnet: "192.0.2.0/24".into(),
            local_subnets: Vec::new(),
            previous_ipv4_forwarding: false,
        };
        let mut objects = vec![
            json!({"table": {"family": "ip", "name": TABLE, "comment": format!("xrat gateway {}", intent.installation_id)}}),
            json!({"chain": {"family": "ip", "table": TABLE, "name": "forward", "type": "filter", "hook": "forward", "prio": -10, "policy": "accept"}}),
            json!({"rule": {"family": "ip", "table": TABLE, "chain": "forward", "expr": [
                {"match": {"op": "==", "left": {"meta": {"key": "iif"}}, "right": "eth0"}},
                {"counter": {"packets": 7, "bytes": 504}},
                {"drop": null}
            ]}}),
        ];

        assert!(matches_closed(&objects, &intent));
        objects[0]["table"]["flags"] = json!(["dormant"]);
        assert!(!matches_closed(&objects, &intent));
        objects[0]["table"].as_object_mut().unwrap().remove("flags");
        objects[2]["rule"]["expr"][2] = json!({"accept": null});
        assert!(!matches_closed(&objects, &intent));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn rejects_invalid_interface_index_before_invoking_nftables() {
        let intent = InstallIntent {
            installation_id: uuid::Uuid::new_v4(),
            lan_interface: String::new(),
            lan_ifindex: 0,
            lan_address: std::net::Ipv4Addr::UNSPECIFIED,
            lan_subnet: String::new(),
            local_subnets: Vec::new(),
            previous_ipv4_forwarding: false,
        };
        assert_eq!(
            install_closed(&intent).unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
    }
}
