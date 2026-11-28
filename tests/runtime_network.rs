#![cfg(target_os = "linux")]
#[path = "runtime_network/dns.rs"]
mod dns;
#[path = "runtime_network/fixtures.rs"]
mod fixtures;
#[path = "runtime_network/gateway_preflight.rs"]
mod gateway_preflight;
#[path = "runtime_network/gateway_topology.rs"]
mod gateway_topology;
#[path = "runtime_network/process.rs"]
mod process;
#[path = "runtime_network/tun.rs"]
mod tun;

#[test]
#[ignore = "requires /dev/net/tun, Linux network namespaces, nftables and pinned engines"]
fn gateway_managed_engines() {
    if process::enter_namespace("gateway_managed_engines") {
        gateway_topology::exercise_managed_engines();
    }
}

#[test]
#[ignore = "requires Linux network namespaces, iproute2, nftables and ping"]
fn gateway_same_interface_topology() {
    if process::enter_namespace("gateway_same_interface_topology") {
        gateway_topology::exercise();
    }
}

#[test]
#[ignore = "gateway namespace fixture worker"]
fn gateway_echo_worker() {
    gateway_topology::echo_worker();
}

#[test]
#[ignore = "gateway namespace fixture worker"]
fn gateway_client_worker() {
    gateway_topology::client_worker();
}

#[test]
#[ignore = "gateway namespace fixture worker"]
fn gateway_closed_client_worker() {
    gateway_topology::closed_client_worker();
}

#[test]
#[ignore = "requires Linux user/network namespaces, iproute2 and nftables"]
fn gateway_preflight_read_only() {
    if process::enter_namespace("gateway_preflight_read_only") {
        gateway_preflight::exercise();
    }
}

#[test]
#[ignore = "requires Linux namespaces, iproute2, OpenSSL and pinned native engines"]
fn dns_network() {
    if process::enter_namespace("dns_network") {
        dns::exercise();
    }
}
#[test]
#[ignore = "requires Linux namespaces, /dev/net/tun, iproute2 and pinned native engines"]
fn tun_network() {
    if process::enter_namespace("tun_network") {
        tun::exercise();
    }
}
#[test]
#[ignore = "private fixture worker launched by tun_network"]
fn fixture_worker() {
    process::assert_isolated();
    tun::serve();
}

#[test]
#[ignore = "private application worker launched by tun_network"]
fn split_traffic_worker() {
    process::assert_isolated();
    // Separate short-lived clients beyond sing-box's one-second process cache.
    std::thread::sleep(std::time::Duration::from_millis(1200));
    fixtures::send("203.0.113.2", false);
}
