use super::*;

pub(super) fn matches_closed(objects: &[Value], intent: &InstallIntent) -> bool {
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
