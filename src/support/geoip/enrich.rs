use std::net::IpAddr;

use super::GeoIpLookup;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EndpointGeoMeta {
    pub location: Option<String>,
    pub country: Option<String>,
    pub asn: Option<String>,
}

impl EndpointGeoMeta {
    pub fn has_lookup_metadata(&self) -> bool {
        self.location.is_some() || self.country.is_some() || self.asn.is_some()
    }
}

pub async fn enrich_address(address: &str, geoip_lookup: &dyn GeoIpLookup) -> EndpointGeoMeta {
    let Some(ip) = resolve_address_ip(address).await else {
        return EndpointGeoMeta::default();
    };

    if let Some(city) = geoip_lookup.city(ip).await.ok().flatten() {
        let country = city.split('/').next().map(str::to_string);
        return EndpointGeoMeta {
            location: Some(city),
            country,
            asn: geoip_lookup.asn(ip).await.ok().flatten(),
        };
    }
    if let Some(country) = geoip_lookup.country(ip).await.ok().flatten() {
        return EndpointGeoMeta {
            location: Some(country.clone()),
            country: Some(country),
            asn: geoip_lookup.asn(ip).await.ok().flatten(),
        };
    }
    if let Some(asn) = geoip_lookup.asn(ip).await.ok().flatten() {
        return EndpointGeoMeta {
            location: Some(asn.clone()),
            country: None,
            asn: Some(asn),
        };
    }

    EndpointGeoMeta::default()
}

pub async fn resolve_address_ip(address: &str) -> Option<IpAddr> {
    let host = address_host(address)?;
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Some(ip);
    }

    tokio::net::lookup_host((host.as_str(), 0))
        .await
        .ok()?
        .map(|socket_addr| socket_addr.ip())
        .next()
}

/// Extract the resolvable host from a config address (`host:port`, a URL, a
/// bracketed IPv6 literal, or a bare host). Used as the GeoIP cache key.
pub fn address_host(address: &str) -> Option<String> {
    let address = address.trim();
    if address.is_empty() {
        return None;
    }

    if let Ok(url) = url::Url::parse(address)
        && let Some(host) = url.host_str()
    {
        return Some(host.to_string());
    }

    if let Ok(socket_addr) = address.parse::<std::net::SocketAddr>() {
        return Some(socket_addr.ip().to_string());
    }

    let without_brackets = address
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'));
    if let Some(host) = without_brackets {
        return Some(host.to_string());
    }

    if let Some((host, port)) = address.rsplit_once(':')
        && !host.contains(':')
        && !port.is_empty()
        && port.chars().all(|char| char.is_ascii_digit())
    {
        return Some(host.to_string());
    }

    Some(address.to_string())
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use super::*;
    use crate::support::geoip::GeoIpError;

    #[derive(Debug)]
    struct TestLookup;

    #[async_trait::async_trait]
    impl GeoIpLookup for TestLookup {
        async fn country(&self, _ip: IpAddr) -> Result<Option<String>, GeoIpError> {
            Ok(Some("ZZ".to_string()))
        }

        async fn city(&self, _ip: IpAddr) -> Result<Option<String>, GeoIpError> {
            Ok(Some("ZZ/Test City".to_string()))
        }

        async fn asn(&self, _ip: IpAddr) -> Result<Option<String>, GeoIpError> {
            Ok(Some("AS64512 TEST".to_string()))
        }

        fn backend_name(&self) -> &'static str {
            "test"
        }
    }

    #[tokio::test]
    async fn enriches_literal_ip_addresses() {
        let meta = enrich_address("8.8.8.8", &TestLookup).await;

        assert_eq!(meta.country.as_deref(), Some("ZZ"));
        assert_eq!(meta.location.as_deref(), Some("ZZ/Test City"));
        assert_eq!(meta.asn.as_deref(), Some("AS64512 TEST"));
    }

    #[test]
    fn extracts_hosts_from_lookup_inputs() {
        assert_eq!(address_host("8.8.8.8").as_deref(), Some("8.8.8.8"));
        assert_eq!(
            address_host("https://google.com:443/path").as_deref(),
            Some("google.com")
        );
        assert_eq!(
            address_host("google.com:443").as_deref(),
            Some("google.com")
        );
        assert_eq!(
            address_host("[2001:4860:4860::8888]").as_deref(),
            Some("2001:4860:4860::8888")
        );
    }
}
