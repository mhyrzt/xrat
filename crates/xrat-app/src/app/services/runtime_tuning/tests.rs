use super::prelude::*;
use super::singbox::*;
use super::xray::*;

use std::collections::BTreeMap;

#[test]
fn parses_xray_version_from_banner() {
    assert_eq!(
        parse_xray_version(
            "Xray 26.3.27 (Xray, Penetrates Everything.) d2758a0 (go1.26.1 linux/amd64)"
        ),
        Some((26, 3, 27))
    );
    assert_eq!(parse_xray_version("Xray 26.7.28"), Some((26, 7, 28)));
    assert_eq!(parse_xray_version("no version here"), None);
}

#[test]
fn default_dns_settings_are_omitted_from_generated_options() {
    let dns = DnsSettings::default();
    let mut xray = XrayGenOptions::default();

    apply_xray_dns_options(&mut xray, &dns).expect("default Xray DNS should be accepted");

    assert!(xray.dns.is_none());
    assert!(
        build_singbox_dns_options(&dns)
            .expect("default sing-box DNS should be accepted")
            .is_none()
    );
}

#[test]
fn xray_dns_mapping_preserves_documented_wire_fields() {
    let mut dns = DnsSettings {
        query_strategy: "UseIPv6".to_string(),
        servers: vec!["8.8.8.8".to_string()],
        use_system_hosts: false,
        disable_cache: true,
        disable_fallback: true,
        enable_parallel_query: false,
        ..Default::default()
    };
    dns.hosts.insert(
        "full:example.test".to_string(),
        DnsHostValue::Many(vec!["192.0.2.10".to_string(), "2001:db8::10".to_string()]),
    );

    let mut options = XrayGenOptions::default();
    apply_xray_dns_options(&mut options, &dns).expect("Xray DNS should map");
    let value = serde_json::to_value(options.dns.expect("DNS output")).unwrap();

    assert_eq!(value["queryStrategy"], "UseIPv6");
    assert_eq!(value["servers"], serde_json::json!(["8.8.8.8"]));
    assert_eq!(value["useSystemHosts"], false);
    assert_eq!(value["disableCache"], true);
    assert_eq!(value["disableFallback"], true);
    assert_eq!(value["enableParallelQuery"], false);
    assert_eq!(
        value["hosts"]["full:example.test"],
        serde_json::json!(["192.0.2.10", "2001:db8::10"])
    );
}

#[test]
fn singbox_dns_mapping_uses_typed_servers_and_safe_fallbacks() {
    let mut dns = DnsSettings {
        query_strategy: "UseIPv4".to_string(),
        servers: vec![
            "8.8.8.8".to_string(),
            "https://dns.google/dns-query".to_string(),
        ],
        use_system_hosts: false,
        disable_cache: true,
        ..Default::default()
    };
    dns.hosts.insert(
        "full:example.test".to_string(),
        DnsHostValue::One("192.0.2.10".to_string()),
    );

    let output = build_singbox_dns_options(&dns)
        .expect("sing-box DNS should map")
        .expect("non-default DNS should be emitted");
    let value = serde_json::to_value(output).unwrap();

    assert_eq!(value["strategy"], "ipv4_only");
    assert_eq!(value["disable_cache"], true);
    assert_eq!(value["final"], "xrat-dns-0");
    assert_eq!(value["servers"][0]["type"], "udp");
    assert_eq!(value["servers"][1]["type"], "https");
    assert_eq!(value["servers"][1]["path"], "/dns-query");
    assert_eq!(
        value["servers"][1]["domain_resolver"],
        SINGBOX_LOCAL_DNS_TAG
    );
    assert_eq!(value["servers"][2]["type"], "local");
    assert_eq!(value["servers"][3]["type"], "hosts");
    assert_eq!(value["servers"][3]["path"], serde_json::json!([]));
    assert_eq!(
        value["rules"][0]["domain"],
        serde_json::json!(["example.test"])
    );
    assert_eq!(value["rules"][0]["action"], "route");
    assert_eq!(value["rules"][0]["server"], SINGBOX_HOSTS_DNS_TAG);
}

#[test]
fn singbox_dns_mapping_rejects_unrepresentable_settings() {
    let mut dns = DnsSettings {
        servers: vec!["1.1.1.1".to_string()],
        ..Default::default()
    };
    let error =
        build_singbox_dns_options(&dns).expect_err("UseSystem must not be silently remapped");
    assert!(
        error
            .to_string()
            .contains("no exact modern sing-box equivalent")
    );

    dns.query_strategy = "UseIPv4".to_string();
    dns.disable_fallback = true;
    let error = build_singbox_dns_options(&dns).expect_err("fallback must be rejected");
    assert!(error.to_string().contains("disable_fallback"));
}

#[test]
fn singbox_dns_mapping_rejects_advanced_hosts_and_unsupported_servers() {
    let mut hosts = BTreeMap::new();
    hosts.insert(
        "domain:example.test".to_string(),
        DnsHostValue::One("192.0.2.10".to_string()),
    );
    let dns = DnsSettings {
        query_strategy: "UseIPv4".to_string(),
        hosts,
        ..Default::default()
    };
    let error = build_singbox_dns_options(&dns).expect_err("domain: keys must be rejected");
    assert!(error.to_string().contains("not an exact hostname"));

    let dns = DnsSettings {
        query_strategy: "UseIPv4".to_string(),
        servers: vec!["h2c://dns.example/dns-query".to_string()],
        ..Default::default()
    };
    let error = build_singbox_dns_options(&dns).expect_err("h2c must be rejected");
    assert!(
        error
            .to_string()
            .contains("no safe modern sing-box mapping")
    );
}
