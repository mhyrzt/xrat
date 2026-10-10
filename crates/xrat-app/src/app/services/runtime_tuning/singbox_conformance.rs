use super::*;
use crate::app::config::*;
use xrat_engines::singbox::{
    SingboxInbound, config_check_command, ensure_supported_binary,
    generate_singbox_runtime_config_with_dns,
};

#[test]
#[ignore = "native DNS conformance skipped by default; requires XRAT_CONFORMANCE_SINGBOX"]
fn native_singbox_dns_conformance() {
    let binary = std::path::PathBuf::from(
        std::env::var_os("XRAT_CONFORMANCE_SINGBOX").expect("explicit validator required"),
    );
    ensure_supported_binary(&binary).unwrap();
    let temporary = tempfile::tempdir().unwrap();
    let directory = std::env::var_os("XRAT_CONFORMANCE_OUTPUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| temporary.path().to_owned());
    std::fs::create_dir_all(&directory).unwrap();
    let node = xrat_config::parse_link("socks5://192.0.2.1:1081")
        .unwrap()
        .unwrap();
    let mut results = Vec::new();
    let mut failures = Vec::new();
    for (kind, address) in [
        ("udp", "udp://192.0.2.2:5353"),
        ("tcp", "tcp://192.0.2.2:5353"),
        ("tls", "tls://resolver.test:853"),
        ("quic", "quic://resolver.test:853"),
        ("https", "https://resolver.test/dns-query"),
        ("h3", "h3://resolver.test/dns-query"),
        ("local", "localhost"),
    ] {
        for fake in [false, true] {
            let legacy = matches!(kind, "quic" | "h3" | "local");
            if legacy && fake {
                continue;
            }
            for tun in [false, true] {
                for strategy in ["UseIPv4", "UseIPv6", "UseIP"] {
                    if legacy && strategy == "UseIP" {
                        continue;
                    }
                    let name = format!("dns-{kind}-fake-{fake}-tun-{tun}-{strategy}");
                    let mut dns = DnsSettings {
                        query_strategy: strategy.into(),
                        use_system_hosts: strategy == "UseIPv4",
                        disable_cache: !fake && strategy == "UseIPv6",
                        resolvers: vec![
                            DnsResolverSettings {
                                tag: "bootstrap".into(),
                                address: "udp://192.0.2.2:5353".into(),
                                path: DnsResolverPath::Bootstrap,
                            },
                            DnsResolverSettings {
                                tag: "remote".into(),
                                address: address.into(),
                                path: if strategy == "UseIPv6" {
                                    DnsResolverPath::Direct
                                } else {
                                    DnsResolverPath::Proxy
                                },
                            },
                        ],
                        bootstrap_resolver: "bootstrap".into(),
                        final_resolver: "remote".into(),
                        listener: DnsListenerSettings {
                            enabled: true,
                            host: "127.0.0.1".into(),
                            port: 1053,
                        },
                        rules: vec![DnsPolicyRule {
                            domain: vec!["direct.test".into()],
                            domain_suffix: vec!["direct-suffix.test".into()],
                            resolver: "bootstrap".into(),
                        }],
                        ..Default::default()
                    };
                    dns.hosts.insert(
                        "resolver.test".into(),
                        DnsHostValue::One("192.0.2.3".into()),
                    );
                    if legacy {
                        dns.servers = vec![address.into()];
                        dns.resolvers.clear();
                        dns.rules.clear();
                        dns.bootstrap_resolver.clear();
                        dns.final_resolver.clear();
                    }
                    dns.fakeip.enabled = fake;
                    dns.fakeip.persist = fake && strategy != "UseIPv6";
                    dns.fakeip.ipv6_range = if strategy == "UseIPv4" {
                        String::new()
                    } else {
                        "fd00:198:18::/96".into()
                    };
                    dns.fakeip.exclude = vec!["excluded.test".into()];
                    dns_validation::validate(&dns, "sing-box", tun)
                        .unwrap_or_else(|error| panic!("{name}: {error}"));
                    let options = build_singbox_dns_options(&dns).unwrap();
                    let mut inbounds =
                        vec![SingboxInbound::socks("socks-in", "127.0.0.1", 1080, None)];
                    if tun {
                        inbounds.push(
                            SingboxInbound::tun(xrat_engines::singbox::SingboxTunOptions {
                                tag: "tun-in".into(),
                                interface_name: "xrat-fixture".into(),
                                address: vec!["172.19.0.1/30".into()],
                                mtu: 1500,
                                stack: "system".into(),
                                auto_route: true,
                                strict_route: true,
                                route_exclude_address: vec![],
                            })
                            .unwrap(),
                        );
                    }
                    let mut config = generate_singbox_runtime_config_with_dns(
                        &node,
                        inbounds,
                        None,
                        None,
                        options.as_ref(),
                    )
                    .unwrap();
                    if tun {
                        config.enable_tun_route();
                    }
                    apply_singbox_dns_runtime(&mut config, &dns, tun, &directory).unwrap();
                    let json = serde_json::to_string_pretty(&config).unwrap();
                    let path = directory.join(format!("{name}.json"));
                    std::fs::write(&path, &json).unwrap();
                    let output = config_check_command(
                        &binary,
                        &path,
                        std::sync::Arc::new(xrat_support::process::SystemProcessSpawner),
                    )
                    .output()
                    .unwrap();
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    results.push(serde_json::json!({"fixture":name,"passed":output.status.success(),"stderr":stderr,"stdout":stdout}));
                    if !output.status.success() {
                        failures.push(format!("{name}: {stdout}\n{stderr}\n{json}"));
                    }
                }
            }
        }
    }
    std::fs::write(
        directory.join("dns-results.json"),
        serde_json::to_vec_pretty(&results).unwrap(),
    )
    .unwrap();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
    println!("native DNS conformance: {} fixtures passed", results.len());
}
