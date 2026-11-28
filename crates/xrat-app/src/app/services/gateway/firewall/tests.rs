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
        boot_id: None,
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
        boot_id: None,
    };
    assert_eq!(
        install_closed(&intent).unwrap_err().kind(),
        std::io::ErrorKind::InvalidInput
    );
}
