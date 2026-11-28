use super::*;
use xrat_support::net::firewall::gateway::{CAPTURE_COMMENT, CAPTURE_DROP_COMMENT, RETURN_COMMENT};

#[cfg(target_os = "linux")]
pub(in crate::app::services::gateway) fn close_capture(
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
pub(in crate::app::services::gateway) fn inspect_active(
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
