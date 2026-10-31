#[cfg(target_os = "linux")]
use std::io;
use std::sync::atomic::Ordering;

use super::*;
#[cfg(target_os = "linux")]
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

#[test]
#[cfg(target_os = "linux")]
fn generates_nftables_ruleset_structure() {
    let mut config = KillSwitchConfig::new("xrat0");
    config.allowed_endpoints.push(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(198, 51, 100, 2)),
        443,
    ));
    config.allowed_endpoints.push(SocketAddr::new(
        IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 2)),
        8443,
    ));
    config.socket_mark = Some(0x255);
    config.allow_lan = true;

    #[cfg(target_os = "linux")]
    {
        let ruleset = nftables::render_ruleset(&config);
        assert!(ruleset.contains("table inet xrat_killswitch"));
        assert!(ruleset.contains("oifname \"lo\" accept"));
        assert!(ruleset.contains("oifname \"xrat0\" accept"));
        assert!(ruleset.contains("meta mark 0x255 accept"));
        assert!(ruleset.contains("ip daddr 198.51.100.2 tcp dport 443 accept"));
        assert!(ruleset.contains("ip daddr 198.51.100.2 udp dport 443 accept"));
        assert!(ruleset.contains("ip6 daddr 2001:db8::2 tcp dport 8443 accept"));
        assert!(ruleset.contains("10.0.0.0/8"));
        assert!(ruleset.contains("192.168.0.0/16"));
        assert!(ruleset.contains("counter drop"));
    }
}

#[test]
fn mock_firewall_lifecycle() {
    let mock = MockFirewallOps::default();
    assert!(!mock.is_active().unwrap());
    assert_eq!(mock.backend_name(), "mock");

    let config = KillSwitchConfig::new("xrat-test");
    mock.apply_killswitch(&config).unwrap();
    assert!(mock.is_active().unwrap());
    assert_eq!(mock.last_config.lock().unwrap().as_ref(), Some(&config));

    mock.remove_killswitch().unwrap();
    assert!(!mock.is_active().unwrap());
    assert!(mock.last_config.lock().unwrap().is_none());
}

#[test]
fn mock_firewall_failure_injection() {
    let mock = MockFirewallOps::default();
    let config = KillSwitchConfig::new("xrat-test");

    mock.fail_apply.store(true, Ordering::SeqCst);
    let apply_err = mock.apply_killswitch(&config).unwrap_err();
    assert_eq!(apply_err.to_string(), "mock firewall apply failure");
    assert!(!mock.is_active().unwrap());

    mock.fail_apply.store(false, Ordering::SeqCst);
    mock.apply_killswitch(&config).unwrap();
    assert!(mock.is_active().unwrap());

    mock.fail_remove.store(true, Ordering::SeqCst);
    let remove_err = mock.remove_killswitch().unwrap_err();
    assert_eq!(remove_err.to_string(), "mock firewall remove failure");
    assert!(mock.is_active().unwrap());

    mock.fail_remove.store(false, Ordering::SeqCst);
    mock.remove_killswitch().unwrap();
    assert!(!mock.is_active().unwrap());
}

#[test]
#[cfg(target_os = "linux")]
fn iptables_is_available_requires_ip6tables_when_ipv6_enabled() {
    // When iptables is missing, always false
    assert!(!iptables::is_available_internal(false, false, false));
    assert!(!iptables::is_available_internal(false, true, false));
    assert!(!iptables::is_available_internal(false, false, true));

    // When iptables is present, but IPv6 is enabled and ip6tables is missing -> false
    assert!(!iptables::is_available_internal(true, false, false));

    // When iptables is present, IPv6 is disabled, even without ip6tables -> true
    assert!(iptables::is_available_internal(true, false, true));

    // When both iptables and ip6tables are present -> true regardless of IPv6 state
    assert!(iptables::is_available_internal(true, true, false));
    assert!(iptables::is_available_internal(true, true, true));
}

#[test]
#[cfg(target_os = "linux")]
fn iptables_apply_v6_rejects_missing_ip6tables_when_ipv6_enabled() {
    let config = KillSwitchConfig::new("xrat0");

    // Missing ip6tables + IPv6 enabled -> NotFound error with fail-closed message
    let err = iptables::apply_v6_staging(&config, false, false).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
    assert_eq!(
        err.to_string(),
        "ip6tables is not available and IPv6 is enabled; cannot establish fail-closed kill switch"
    );

    // Missing ip6tables + IPv6 disabled -> Ok
    let res = iptables::apply_v6_staging(&config, false, true);
    assert!(res.is_ok());
}

#[test]
#[cfg(target_os = "linux")]
fn iptables_is_ipv6_disabled_inspects_proc_files() {
    let dir = tempfile::tempdir().unwrap();
    let sysctl_path = dir.path().join("disable_ipv6");
    let if_inet6_path = dir.path().join("if_inet6");

    // Case 1: sysctl says "1" -> IPv6 disabled
    std::fs::write(&sysctl_path, "1\n").unwrap();
    std::fs::write(
        &if_inet6_path,
        "00000000000000000000000000000001 01 80 10 80       lo\n",
    )
    .unwrap();
    assert!(iptables::is_ipv6_disabled_at(&sysctl_path, &if_inet6_path));

    // Case 2: sysctl says "0", if_inet6 has interfaces -> IPv6 enabled (is_ipv6_disabled is false)
    std::fs::write(&sysctl_path, "0\n").unwrap();
    assert!(!iptables::is_ipv6_disabled_at(&sysctl_path, &if_inet6_path));

    // Case 3: sysctl says "0", if_inet6 is empty -> IPv6 disabled
    std::fs::write(&if_inet6_path, "  \n").unwrap();
    assert!(iptables::is_ipv6_disabled_at(&sysctl_path, &if_inet6_path));

    // Case 4: if_inet6 does not exist -> IPv6 disabled
    std::fs::remove_file(&if_inet6_path).unwrap();
    assert!(iptables::is_ipv6_disabled_at(&sysctl_path, &if_inet6_path));

    // Case 5: neither exists -> IPv6 disabled
    std::fs::remove_file(&sysctl_path).unwrap();
    assert!(iptables::is_ipv6_disabled_at(&sysctl_path, &if_inet6_path));
}

#[test]
#[cfg(target_os = "linux")]
fn iptables_run_iptables_propagates_mutation_failures() {
    // Non-zero exit code with invalid flag captures stderr
    let err = iptables::run_iptables("iptables", &["--invalid-test-flag-xyz"]).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::Other);
    let msg = err.to_string();
    assert!(msg.contains("iptables failed"));
    assert!(msg.contains("unrecognised option") || msg.contains("invalid"));

    // Non-existent binary returns NotFound
    let not_found = iptables::run_iptables("nonexistent_binary_xyz_12345", &[]).unwrap_err();
    assert_eq!(not_found.kind(), io::ErrorKind::NotFound);
}

#[test]
#[cfg(target_os = "linux")]
#[ignore = "requires Linux namespaces and privileges"]
fn real_linux_nftables_lifecycle_in_isolated_netns() {
    if !nftables::is_available() {
        return;
    }
    let status = std::process::Command::new("unshare")
        .args([
            "--user",
            "--map-root-user",
            "--net",
            "sh",
            "-c",
            r#"
            set -e
            nft -f - << 'EOF'
add table inet xrat_killswitch
flush table inet xrat_killswitch
table inet xrat_killswitch {
    chain output {
        type filter hook output priority -10; policy drop;
        oifname "lo" accept
        oifname "xrat0" accept
        meta mark 0x000000ff accept
        ip daddr 203.0.113.2 tcp dport 443 accept
        udp sport 68 udp dport 67 accept
        counter drop
    }
}
EOF
            nft list table inet xrat_killswitch > /dev/null
            nft delete table inet xrat_killswitch
            ! nft list table inet xrat_killswitch 2> /dev/null
        "#,
        ])
        .status();
    if let Ok(status) = status {
        assert!(status.success(), "isolated netns nftables test failed");
    }
}

#[test]
#[cfg(target_os = "linux")]
#[ignore = "requires Linux namespaces and privileges"]
fn real_linux_iptables_lifecycle_in_isolated_netns() {
    if find_in_path("iptables").is_none() {
        return;
    }
    let status = std::process::Command::new("unshare")
        .args([
            "--user",
            "--map-root-user",
            "--net",
            "sh",
            "-c",
            r#"
            set -e
            iptables -N XRAT_KS_STG
            iptables -A XRAT_KS_STG -o lo -j ACCEPT
            iptables -A XRAT_KS_STG -j DROP
            iptables -E XRAT_KS_STG XRAT_KS
            iptables -I OUTPUT 1 -j XRAT_KS
            iptables -C OUTPUT -j XRAT_KS

            # Atomic swap using staging chain
            iptables -N XRAT_KS_STG
            iptables -A XRAT_KS_STG -o lo -j ACCEPT
            iptables -A XRAT_KS_STG -j DROP
            iptables -I OUTPUT 1 -j XRAT_KS_STG
            iptables -D OUTPUT -j XRAT_KS
            iptables -F XRAT_KS
            iptables -X XRAT_KS
            iptables -E XRAT_KS_STG XRAT_KS
            iptables -C OUTPUT -j XRAT_KS

            # Cleanup
            iptables -D OUTPUT -j XRAT_KS
            iptables -F XRAT_KS
            iptables -X XRAT_KS
            ! iptables -C OUTPUT -j XRAT_KS 2> /dev/null
        "#,
        ])
        .status();
    if let Ok(status) = status {
        assert!(status.success(), "isolated netns iptables test failed");
    }
}
