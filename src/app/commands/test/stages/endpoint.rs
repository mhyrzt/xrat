use super::*;
use std::net::IpAddr;

use crate::support::geoip::GeoIpLookup;

pub(crate) fn classify_endpoint_location(endpoint_ip: Option<&str>) -> Option<String> {
    geoip::classify_endpoint_location(endpoint_ip)
}

pub(crate) struct EndpointMeta {
    pub(crate) location: Option<String>,
    pub(crate) country: Option<String>,
    pub(crate) asn: Option<String>,
}

pub(crate) async fn resolve_endpoint_meta(
    endpoint_ip: Option<&str>,
    geoip_enabled: bool,
    geoip_lookup: &dyn GeoIpLookup,
) -> EndpointMeta {
    if geoip_enabled && let Some(ip) = endpoint_ip.and_then(|value| value.parse::<IpAddr>().ok()) {
        if let Some(city) = geoip_lookup.city(ip).await.ok().flatten() {
            let country = city.split('/').next().map(str::to_string);
            return EndpointMeta {
                location: Some(city),
                country,
                asn: geoip_lookup.asn(ip).await.ok().flatten(),
            };
        }
        if let Some(country) = geoip_lookup.country(ip).await.ok().flatten() {
            return EndpointMeta {
                location: Some(country.clone()),
                country: Some(country),
                asn: geoip_lookup.asn(ip).await.ok().flatten(),
            };
        }
        if let Some(asn) = geoip_lookup.asn(ip).await.ok().flatten() {
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
