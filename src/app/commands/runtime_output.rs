use std::net::{IpAddr, UdpSocket};

pub fn format_inbound_endpoint(host: &str, port: u16) -> String {
    format_inbound_endpoint_with_ip(host, port, local_machine_ip)
}

fn format_inbound_endpoint_with_ip<F>(host: &str, port: u16, ip_lookup: F) -> String
where
    F: FnOnce() -> Option<IpAddr>,
{
    if host != "0.0.0.0" {
        return format!("{host}:{port}");
    }

    // A wildcard bind is not a useful copy/paste address, so substitute the
    // reachable host IP directly into the endpoint when it can be resolved.
    ip_lookup()
        .map(|ip| format!("{ip}:{port}"))
        .unwrap_or_else(|| format!("{host}:{port}"))
}

fn local_machine_ip() -> Option<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip())
}

#[cfg(test)]
mod tests {
    use super::format_inbound_endpoint_with_ip;

    #[test]
    fn substitutes_host_ip_for_wildcard_bind() {
        let endpoint = format_inbound_endpoint_with_ip("0.0.0.0", 1080, || {
            Some("192.0.2.10".parse().expect("test ip should parse"))
        });

        assert_eq!(endpoint, "192.0.2.10:1080");
    }

    #[test]
    fn leaves_specific_bind_unchanged() {
        let endpoint = format_inbound_endpoint_with_ip("127.0.0.1", 1080, || {
            Some("192.0.2.10".parse().expect("test ip should parse"))
        });

        assert_eq!(endpoint, "127.0.0.1:1080");
    }

    #[test]
    fn leaves_wildcard_unchanged_when_host_ip_is_unknown() {
        let endpoint = format_inbound_endpoint_with_ip("0.0.0.0", 1080, || None);

        assert_eq!(endpoint, "0.0.0.0:1080");
    }
}
