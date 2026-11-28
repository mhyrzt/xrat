use std::io;

use serde_json::{Value, json};

use super::{SystemTunInterfaceOps, TunInterfaceOps};

pub const TABLE: &str = "420";
pub const PRIORITY: &str = "4200";
const PROTOCOL: &str = "201";

/// Install ingress-only capture routing behind an already verified forwarding
/// guard. The terminal route remains if the kernel removes the TUN route.
pub fn install(lan: &str, subnet: &str, capture: &str) -> io::Result<()> {
    install_with_local(lan, subnet, capture, &[])
}

pub fn install_with_local(
    lan: &str,
    subnet: &str,
    capture: &str,
    local: &[String],
) -> io::Result<()> {
    preflight_install()?;

    ip(&[
        "route",
        "add",
        "unreachable",
        "default",
        "metric",
        "32767",
        "table",
        TABLE,
        "proto",
        PROTOCOL,
    ])?;
    ip(&[
        "route", "add", "default", "dev", capture, "table", TABLE, "proto", PROTOCOL,
    ])?;
    for destination in local {
        ip(&[
            "route",
            "add",
            destination,
            "dev",
            lan,
            "table",
            TABLE,
            "proto",
            PROTOCOL,
        ])?;
    }
    ip(&[
        "rule", "add", "priority", PRIORITY, "iif", lan, "from", subnet, "lookup", TABLE,
        "protocol", PROTOCOL,
    ])?;
    Ok(())
}

pub fn preflight_install() -> io::Result<()> {
    verify_precedence(false)?;
    if !table_routes()?.is_empty() || list_rules()?.iter().any(references_table) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "gateway capture route table is already in use",
        ));
    }
    Ok(())
}

fn verify_precedence(replacing_owned_rule: bool) -> io::Result<()> {
    let highest = if replacing_owned_rule { 4199 } else { 4200 };
    let policy = SystemTunInterfaceOps.policy_rules()?;
    if policy.iter().any(|rule| {
        rule.family == 2
            && rule
                .priority()
                .is_some_and(|priority| (1..=highest).contains(&priority))
    }) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "an IPv4 policy rule can take precedence over gateway capture",
        ));
    }
    Ok(())
}

pub fn preflight_replacement(
    lan: &str,
    subnet: &str,
    capture: &str,
    local: &[String],
) -> io::Result<()> {
    verify_precedence(true)?;
    check_or_remove(lan, subnet, capture, local, false)
}

/// Remove only the expected route and rule shape while the closed guard remains.
pub fn remove(lan: &str, subnet: &str, capture: &str) -> io::Result<()> {
    remove_with_local(lan, subnet, capture, &[])
}

pub fn remove_with_local(
    lan: &str,
    subnet: &str,
    capture: &str,
    local: &[String],
) -> io::Result<()> {
    check_or_remove(lan, subnet, capture, local, true)
}

fn check_or_remove(
    lan: &str,
    subnet: &str,
    capture: &str,
    local: &[String],
    remove: bool,
) -> io::Result<()> {
    let (network, prefix) = subnet
        .split_once('/')
        .and_then(|(network, prefix)| {
            Some((
                network.parse::<std::net::Ipv4Addr>().ok()?,
                prefix.parse::<u8>().ok()?,
            ))
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid LAN subnet"))?;
    let expected_rule = json!({
        "priority": 4200, "src": network.to_string(), "srclen": prefix,
        "iif": lan, "table": TABLE, "protocol": PROTOCOL
    });
    let capture_route = json!({
        "dst": "default", "dev": capture, "table": TABLE, "protocol": PROTOCOL,
        "scope": "253", "flags": []
    });
    let terminal_route = json!({
        "type": "7", "dst": "default", "table": TABLE, "protocol": PROTOCOL,
        "metric": 32767, "flags": []
    });
    let local_routes = local
        .iter()
        .map(|destination| {
            json!({
                "dst": destination, "dev": lan, "table": TABLE, "protocol": PROTOCOL,
                "scope": "253", "flags": []
            })
        })
        .collect::<Vec<_>>();
    let rules = list_rules()?;
    let claiming: Vec<_> = rules
        .iter()
        .filter(|rule| rule["priority"] == 4200 || references_table(rule))
        .collect();
    let routes = table_routes()?;
    if claiming
        .as_slice()
        .iter()
        .any(|rule| **rule != expected_rule)
        || claiming.len() > 1
        || routes.iter().any(|route| {
            *route != capture_route && *route != terminal_route && !local_routes.contains(route)
        })
        || routes.len() > 2 + local_routes.len()
    {
        return Err(io::Error::other(
            "gateway policy rule or route table differs from the owned shape",
        ));
    }

    if !remove {
        return Ok(());
    }
    if !claiming.is_empty() {
        ip(&[
            "rule", "delete", "priority", PRIORITY, "iif", lan, "from", subnet, "lookup", TABLE,
            "protocol", PROTOCOL,
        ])?;
    }
    if routes.contains(&capture_route) {
        ip(&[
            "route", "delete", "default", "dev", capture, "table", TABLE, "proto", PROTOCOL,
        ])?;
    }
    for (subnet, route) in local.iter().zip(&local_routes) {
        if routes.contains(route) {
            ip(&[
                "route", "delete", subnet, "dev", lan, "table", TABLE, "proto", PROTOCOL,
            ])?;
        }
    }
    if routes.contains(&terminal_route) {
        ip(&[
            "route",
            "delete",
            "unreachable",
            "default",
            "metric",
            "32767",
            "table",
            TABLE,
            "proto",
            PROTOCOL,
        ])?;
    }
    Ok(())
}

fn list_rules() -> io::Result<Vec<Value>> {
    Ok(serde_json::from_slice(&ip(&["-j", "-N", "rule", "show"])?)?)
}

fn references_table(object: &Value) -> bool {
    object["table"] == TABLE || object["table"] == 420
}

fn table_routes() -> io::Result<Vec<Value>> {
    let routes: Vec<Value> =
        serde_json::from_slice(&ip(&["-j", "-N", "route", "show", "table", "all"])?)?;
    Ok(routes.into_iter().filter(references_table).collect())
}

fn ip(arguments: &[&str]) -> io::Result<Vec<u8>> {
    let output = crate::process::Command::new("ip")
        .arg("-4")
        .args(arguments)
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "ip {} failed: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(output.stdout)
}
