use super::*;

pub(crate) fn classify_endpoint_location(endpoint_ip: Option<&str>) -> Option<String> {
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

pub(crate) struct EndpointMeta {
    pub(crate) location: Option<String>,
    pub(crate) country: Option<String>,
    pub(crate) asn: Option<String>,
}

pub(crate) fn resolve_endpoint_meta(
    endpoint_ip: Option<&str>,
    geoip_enabled: bool,
    geoip_city_path: &std::path::Path,
    geoip_country_path: &std::path::Path,
    geoip_asn_path: &std::path::Path,
) -> EndpointMeta {
    if geoip_enabled && let Some(ip) = endpoint_ip {
        if let Some(city) = geoip::lookup_city_label(geoip_city_path, ip) {
            let country = city.split('/').next().map(str::to_string);
            return EndpointMeta {
                location: Some(city),
                country,
                asn: geoip::lookup_asn_label(geoip_asn_path, ip),
            };
        }
        if let Some(country) = geoip::lookup_country_iso(geoip_country_path, ip) {
            return EndpointMeta {
                location: Some(country.clone()),
                country: Some(country),
                asn: geoip::lookup_asn_label(geoip_asn_path, ip),
            };
        }
        if let Some(asn) = geoip::lookup_asn_label(geoip_asn_path, ip) {
            return EndpointMeta {
                location: Some(asn.clone()),
                country: None,
                asn: Some(asn),
            };
        }
    }
    EndpointMeta {
        location: classify_endpoint_location(endpoint_ip),
        country: None,
        asn: None,
    }
}
