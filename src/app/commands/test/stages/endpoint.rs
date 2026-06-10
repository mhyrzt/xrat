use super::*;

use crate::support::geoip::{EndpointGeoMeta, GeoIpLookup};

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
    if geoip_enabled && let Some(endpoint_ip) = endpoint_ip {
        let meta = geoip::enrich_address(endpoint_ip, geoip_lookup).await;
        if meta != EndpointGeoMeta::default() {
            return EndpointMeta {
                location: meta.location,
                country: meta.country,
                asn: meta.asn,
            };
        }
    }
    EndpointMeta {
        location: classify_endpoint_location(endpoint_ip),
        country: None,
        asn: None,
    }
}
