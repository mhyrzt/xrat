use std::net::SocketAddr;

/// Configuration parameters for establishing a fail-closed Kill Switch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KillSwitchConfig {
    /// Interface name of the TUN device (e.g. `xrat0`).
    pub tun_interface: String,
    /// Destination endpoints explicitly allowed for proxy handshakes and bootstrap DNS.
    pub allowed_endpoints: Vec<SocketAddr>,
    /// fwmark / socket mark applied by the proxy core (e.g. `0x255`).
    pub socket_mark: Option<u32>,
    /// Whether to permit RFC 1918 / IPv6 ULA local area network communication.
    pub allow_lan: bool,
    /// Whether to block all IPv6 egress.
    pub block_ipv6: bool,
}

impl KillSwitchConfig {
    pub fn new(tun_interface: impl Into<String>) -> Self {
        Self {
            tun_interface: tun_interface.into(),
            allowed_endpoints: Vec::new(),
            socket_mark: None,
            allow_lan: true,
            block_ipv6: false,
        }
    }
}
