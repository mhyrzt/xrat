use std::net::IpAddr;

pub fn classify_endpoint_location(endpoint_ip: Option<&str>) -> Option<String> {
    let ip = endpoint_ip?.parse::<IpAddr>().ok()?;
    let label = match ip {
        IpAddr::V4(v4) => {
            if v4.is_private() {
                "private_ipv4"
            } else if v4.is_loopback() {
                "loopback_ipv4"
            } else if v4.is_link_local() {
                "link_local_ipv4"
            } else {
                "public"
            }
        }
        IpAddr::V6(v6) => {
            if v6.is_loopback() {
                "loopback_ipv6"
            } else if v6.is_unique_local() {
                "unique_local_ipv6"
            } else if v6.is_unicast_link_local() {
                "link_local_ipv6"
            } else {
                "public"
            }
        }
    };
    Some(label.to_string())
}

#[cfg(test)]
mod tests {
    use super::classify_endpoint_location;

    #[test]
    fn classifies_public_and_private_ip_labels() {
        assert_eq!(
            classify_endpoint_location(Some("8.8.8.8")).as_deref(),
            Some("public")
        );
        assert_eq!(
            classify_endpoint_location(Some("10.0.0.1")).as_deref(),
            Some("private_ipv4")
        );
        assert_eq!(
            classify_endpoint_location(Some("::1")).as_deref(),
            Some("loopback_ipv6")
        );
        assert!(classify_endpoint_location(Some("not-an-ip")).is_none());
    }
}
