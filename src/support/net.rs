pub fn connect_host_for_bind_host(host: &str) -> String {
    match host {
        "0.0.0.0" => "127.0.0.1".to_string(),
        "::" => "::1".to_string(),
        _ => host.to_string(),
    }
}

/// Best-effort primary LAN IP of this host. Opens a UDP socket toward a public
/// address (no packets are sent) so the OS picks the outbound interface, then
/// reads back its local address. Returns `None` when no route can be resolved.
pub fn primary_local_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

/// Resolve a network interface name to a bindable address, preferring IPv4.
/// Used to turn an inbound `listen_interface` setting into a concrete `listen`
/// address. Returns `None` when the interface is unknown or has no address.
pub fn interface_address(name: &str) -> Option<String> {
    let addrs = if_addrs::get_if_addrs().ok()?;
    let mut ipv6: Option<String> = None;
    for iface in addrs {
        if iface.name != name {
            continue;
        }
        let ip = iface.ip();
        if ip.is_ipv4() {
            return Some(ip.to_string());
        }
        ipv6.get_or_insert_with(|| ip.to_string());
    }
    ipv6
}
