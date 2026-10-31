#[path = "net/capture.rs"]
mod capture;
pub use capture::{NetworkAdapter, NetworkRoute, NetworkSnapshot};

#[path = "net/firewall/mod.rs"]
pub mod firewall;
pub use firewall::{FirewallOps, KillSwitchConfig, MockFirewallOps, SystemFirewallOps};

pub fn format_host_port(host: &str, port: u16) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

pub fn format_url_authority(host: &str, port: u16) -> String {
    format_host_port(&host.replace('%', "%25"), port)
}

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
pub trait LocalIpResolver: Send + Sync {
    fn primary_ip(&self) -> Option<std::net::IpAddr>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemLocalIpResolver;
impl LocalIpResolver for SystemLocalIpResolver {
    fn primary_ip(&self) -> Option<std::net::IpAddr> {
        let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
        socket.connect("8.8.8.8:80").ok()?;
        socket.local_addr().ok().map(|addr| addr.ip())
    }
}

pub fn primary_local_ip() -> Option<String> {
    SystemLocalIpResolver.primary_ip().map(|ip| ip.to_string())
}

/// Resolve a host/port to an IP address. Used for DNS pre-resolution in TUN mode
/// to break routing loops before tunnel creation.
pub trait HostResolver: Send + Sync {
    fn resolve(&self, host: &str, port: u16) -> Option<std::net::IpAddr>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemHostResolver;

impl HostResolver for SystemHostResolver {
    fn resolve(&self, host: &str, port: u16) -> Option<std::net::IpAddr> {
        use std::net::ToSocketAddrs;
        (host, port)
            .to_socket_addrs()
            .ok()?
            .next()
            .map(|addr| addr.ip())
    }
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

/// Whether a network interface with `name` currently exists. Checks the Linux
/// sysfs view first so interfaces without an assigned address are still
/// detected, then falls back to the cross-platform address list.
pub fn interface_exists(name: &str) -> bool {
    let name = name.trim();
    if name.is_empty() {
        return false;
    }
    if std::path::Path::new("/sys/class/net").join(name).exists() {
        return true;
    }
    if_addrs::get_if_addrs()
        .map(|addrs| addrs.iter().any(|iface| iface.name == name))
        .unwrap_or(false)
}

/// Information about a network interface detected in the system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelInterfaceInfo {
    pub name: String,
    pub ifindex: u32,
    pub is_tun: bool,
}

/// Operations on TUN interfaces, abstracting kernel interface inspection and
/// deletion so unit tests can run deterministically without modifying host devices.
pub trait TunInterfaceOps: Send + Sync {
    fn inspect_interface(&self, name: &str) -> std::io::Result<Option<KernelInterfaceInfo>>;
    fn delete_interface(&self, name: &str, expected_ifindex: u32) -> std::io::Result<()>;
    fn interface_identity(&self, _name: &str) -> std::io::Result<Option<String>> {
        Ok(None)
    }
    fn policy_rules(&self) -> std::io::Result<Vec<KernelPolicyRule>> {
        Ok(vec![])
    }
    fn delete_policy_rule(&self, _rule: &KernelPolicyRule) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "policy rule deletion is unavailable",
        ))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct KernelPolicyRule {
    pub family: u8,
    pub destination_prefix: u8,
    pub source_prefix: u8,
    #[serde(default)]
    pub tos: u8,
    pub table: u8,
    pub action: u8,
    pub flags: u32,
    pub attributes: Vec<KernelRuleAttribute>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct KernelRuleAttribute {
    pub kind: u16,
    pub value: Vec<u8>,
}

impl KernelPolicyRule {
    pub fn priority(&self) -> Option<u32> {
        self.attributes
            .iter()
            .find(|attribute| attribute.kind == 6)
            .and_then(|attribute| attribute.value.as_slice().try_into().ok())
            .map(u32::from_ne_bytes)
    }
    pub fn is_singbox_capture_rule(&self) -> bool {
        matches!(self.family, 2 | 10)
            && self
                .priority()
                .is_some_and(|priority| (9000..=9010).contains(&priority))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemTunInterfaceOps;

impl TunInterfaceOps for SystemTunInterfaceOps {
    fn interface_identity(&self, name: &str) -> std::io::Result<Option<String>> {
        #[cfg(windows)]
        {
            windows_links::identity(name)
        }
        #[cfg(not(windows))]
        {
            let _ = name;
            Ok(None)
        }
    }
    fn policy_rules(&self) -> std::io::Result<Vec<KernelPolicyRule>> {
        #[cfg(target_os = "linux")]
        {
            kernel_rules::list()
        }
        #[cfg(not(target_os = "linux"))]
        {
            Ok(vec![])
        }
    }

    fn delete_policy_rule(&self, rule: &KernelPolicyRule) -> std::io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            kernel_rules::delete(rule)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = rule;
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Linux policy rules only",
            ))
        }
    }
    fn inspect_interface(&self, name: &str) -> std::io::Result<Option<KernelInterfaceInfo>> {
        let name = name.trim();
        if name.is_empty() || name.contains('/') || name == "." || name == ".." {
            return Ok(None);
        }
        #[cfg(target_os = "linux")]
        {
            kernel_links::inspect(name)
        }
        #[cfg(target_os = "macos")]
        {
            macos_links::inspect(name)
        }
        #[cfg(windows)]
        {
            windows_links::inspect(name)
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
        {
            if interface_exists(name) {
                Ok(Some(KernelInterfaceInfo {
                    name: name.to_string(),
                    ifindex: 0,
                    is_tun: false,
                }))
            } else {
                Ok(None)
            }
        }
    }

    fn delete_interface(&self, name: &str, expected_ifindex: u32) -> std::io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            let _ = name;
            linux_netlink::delete_link_by_index(expected_ifindex)
        }
        #[cfg(target_os = "macos")]
        {
            macos_links::wait_for_release(name, expected_ifindex)
        }
        #[cfg(windows)]
        {
            windows_links::wait_for_release(name, expected_ifindex)
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
        {
            let _ = (name, expected_ifindex);
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "in-process TUN interface deletion is only supported on Linux",
            ))
        }
    }
}

#[cfg(target_os = "linux")]
#[path = "net/link.rs"]
mod kernel_links;

#[cfg(target_os = "macos")]
#[path = "net/macos.rs"]
mod macos_links;

#[cfg(windows)]
#[path = "net/windows.rs"]
mod windows_links;

#[cfg(any(target_os = "macos", test))]
#[path = "net/macos_routes.rs"]
mod macos_routes;

#[cfg(target_os = "macos")]
pub fn preflight_macos_capture(
    addresses: &[String],
    owned: Option<(&str, u32)>,
) -> std::io::Result<()> {
    macos_routes::preflight(addresses, owned)
}

#[cfg(target_os = "macos")]
pub fn macos_capture_ready(addresses: &[String], interface: &str) -> std::io::Result<bool> {
    macos_routes::ready(addresses, interface)
}

#[cfg(target_os = "linux")]
#[path = "net/rules.rs"]
mod kernel_rules;

#[cfg(target_os = "linux")]
mod linux_netlink {
    #[repr(C)]
    struct Nlmsghdr {
        nlmsg_len: u32,
        nlmsg_type: u16,
        nlmsg_flags: u16,
        nlmsg_seq: u32,
        nlmsg_pid: u32,
    }

    #[repr(C)]
    struct Ifinfomsg {
        ifi_family: u8,
        __ifi_pad: u8,
        ifi_type: u16,
        ifi_index: i32,
        ifi_flags: u32,
        ifi_change: u32,
    }

    unsafe extern "C" {
        fn socket(domain: i32, type_: i32, protocol: i32) -> i32;
        fn send(fd: i32, buf: *const u8, len: usize, flags: i32) -> isize;
        fn recv(fd: i32, buf: *mut u8, len: usize, flags: i32) -> isize;
        fn close(fd: i32) -> i32;
    }

    const AF_NETLINK: i32 = 16;
    const SOCK_RAW: i32 = 3;
    const SOCK_CLOEXEC: i32 = 0x80000;
    const NETLINK_ROUTE: i32 = 0;
    const RTM_DELLINK: u16 = 17;
    const NLM_F_REQUEST: u16 = 1;
    const NLM_F_ACK: u16 = 4;
    const NLMSG_ERROR: u16 = 2;

    pub fn delete_link_by_index(ifindex: u32) -> std::io::Result<()> {
        let fd = unsafe { socket(AF_NETLINK, SOCK_RAW | SOCK_CLOEXEC, NETLINK_ROUTE) };
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        struct FdGuard(i32);
        impl Drop for FdGuard {
            fn drop(&mut self) {
                unsafe { close(self.0) };
            }
        }
        let _guard = FdGuard(fd);

        let req_len = (std::mem::size_of::<Nlmsghdr>() + std::mem::size_of::<Ifinfomsg>()) as u32;
        #[repr(C)]
        struct DellinkRequest {
            hdr: Nlmsghdr,
            ifi: Ifinfomsg,
        }

        let req = DellinkRequest {
            hdr: Nlmsghdr {
                nlmsg_len: req_len,
                nlmsg_type: RTM_DELLINK,
                nlmsg_flags: NLM_F_REQUEST | NLM_F_ACK,
                nlmsg_seq: 1,
                nlmsg_pid: 0,
            },
            ifi: Ifinfomsg {
                ifi_family: 0,
                __ifi_pad: 0,
                ifi_type: 0,
                ifi_index: ifindex as i32,
                ifi_flags: 0,
                ifi_change: 0,
            },
        };

        let bytes = unsafe {
            std::slice::from_raw_parts(
                &req as *const _ as *const u8,
                std::mem::size_of::<DellinkRequest>(),
            )
        };

        let sent = unsafe { send(fd, bytes.as_ptr(), bytes.len(), 0) };
        if sent < 0 {
            return Err(std::io::Error::last_os_error());
        }

        let mut buf = [0u8; 1024];
        let n = unsafe { recv(fd, buf.as_mut_ptr(), buf.len(), 0) };
        if n < 0 {
            return Err(std::io::Error::last_os_error());
        }
        if (n as usize) < std::mem::size_of::<Nlmsghdr>() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "truncated netlink response",
            ));
        }

        let hdr = unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const Nlmsghdr) };
        if hdr.nlmsg_type == NLMSG_ERROR {
            if (n as usize) < std::mem::size_of::<Nlmsghdr>() + 4 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "truncated netlink error",
                ));
            }
            let err_code = unsafe {
                std::ptr::read_unaligned(
                    buf.as_ptr().add(std::mem::size_of::<Nlmsghdr>()) as *const i32
                )
            };
            if err_code == 0 {
                return Ok(());
            }
            return Err(std::io::Error::from_raw_os_error(-err_code));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{format_host_port, format_url_authority, interface_exists};

    #[test]
    fn formats_ipv4_dns_and_ipv6_authorities_without_double_bracketing() {
        for (host, expected) in [
            ("127.0.0.1", "127.0.0.1:1080"),
            ("localhost", "localhost:1080"),
            ("::1", "[::1]:1080"),
            ("[::1]", "[::1]:1080"),
        ] {
            assert_eq!(format_host_port(host, 1080), expected);
            assert_eq!(format_url_authority(host, 1080), expected);
        }
        assert_eq!(format_host_port("fe80::1%3", 1080), "[fe80::1%3]:1080");
        assert_eq!(
            format_url_authority("fe80::1%3", 1080),
            "[fe80::1%253]:1080"
        );
    }

    #[test]
    fn detects_missing_interfaces() {
        assert!(!interface_exists("xrat-missing-interface"));
        assert!(!interface_exists("  "));
    }

    #[cfg(unix)]
    #[test]
    fn detects_loopback_interface() {
        assert!(interface_exists(if cfg!(target_os = "macos") {
            "lo0"
        } else {
            "lo"
        }));
    }
}

#[cfg(windows)]
#[path = "net/windows_snapshot.rs"]
mod windows_snapshot;
#[cfg(windows)]
pub use windows_snapshot::snapshot as windows_network_snapshot;
