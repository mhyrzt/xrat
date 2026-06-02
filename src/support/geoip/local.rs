use std::net::IpAddr;
use std::path::{Path, PathBuf};

use maxminddb::{Reader, geoip2};

use super::{GeoIpError, GeoIpLookup};

#[derive(Clone, Debug)]
pub struct LocalMmdbLookup {
    country_path: PathBuf,
    city_path: PathBuf,
    asn_path: PathBuf,
}

impl LocalMmdbLookup {
    pub fn new(country_path: PathBuf, city_path: PathBuf, asn_path: PathBuf) -> Self {
        Self {
            country_path,
            city_path,
            asn_path,
        }
    }

    fn lookup_country_iso_ip(mmdb_path: &Path, ip_addr: IpAddr) -> Option<String> {
        let reader = Reader::open_readfile(mmdb_path).ok()?;
        let country: geoip2::Country<'_> = reader.lookup(ip_addr).ok()??;

        country
            .country
            .and_then(|country| country.iso_code)
            .map(|code| code.to_string())
            .or_else(|| {
                country
                    .registered_country
                    .and_then(|country| country.iso_code)
                    .map(|code| code.to_string())
            })
    }

    fn lookup_city_label_ip(mmdb_path: &Path, ip_addr: IpAddr) -> Option<String> {
        let reader = Reader::open_readfile(mmdb_path).ok()?;
        let city: geoip2::City<'_> = reader.lookup(ip_addr).ok()??;

        let country = city
            .country
            .and_then(|country| country.iso_code)
            .map(str::to_string);
        let region = city
            .subdivisions
            .as_ref()
            .and_then(|subs| subs.first())
            .and_then(|sub| sub.names.as_ref())
            .and_then(|names| names.get("en").copied())
            .map(str::to_string);
        let city_name = city
            .city
            .and_then(|city| city.names)
            .and_then(|names| names.get("en").copied())
            .map(str::to_string);

        match (country, region, city_name) {
            (Some(cc), Some(region), Some(city)) => Some(format!("{cc}/{region}/{city}")),
            (Some(cc), _, Some(city)) => Some(format!("{cc}/{city}")),
            (Some(cc), _, _) => Some(cc),
            _ => None,
        }
    }

    fn lookup_asn_label_ip(mmdb_path: &Path, ip_addr: IpAddr) -> Option<String> {
        let reader = Reader::open_readfile(mmdb_path).ok()?;
        let asn: geoip2::Asn<'_> = reader.lookup(ip_addr).ok()??;
        let number = asn.autonomous_system_number?;
        let org = asn.autonomous_system_organization.unwrap_or("UNKNOWN");
        Some(format!("AS{number} {org}"))
    }
}

#[async_trait::async_trait]
impl GeoIpLookup for LocalMmdbLookup {
    async fn country(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        Ok(Self::lookup_country_iso_ip(&self.country_path, ip))
    }

    async fn city(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        Ok(Self::lookup_city_label_ip(&self.city_path, ip))
    }

    async fn asn(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        Ok(Self::lookup_asn_label_ip(&self.asn_path, ip))
    }

    fn backend_name(&self) -> &'static str {
        "mmdb"
    }
}

pub fn lookup_country_iso(mmdb_path: &Path, ip: &str) -> Option<String> {
    let ip_addr: IpAddr = ip.parse().ok()?;
    LocalMmdbLookup::lookup_country_iso_ip(mmdb_path, ip_addr)
}

pub fn lookup_city_label(mmdb_path: &Path, ip: &str) -> Option<String> {
    let ip_addr: IpAddr = ip.parse().ok()?;
    LocalMmdbLookup::lookup_city_label_ip(mmdb_path, ip_addr)
}

pub fn lookup_asn_label(mmdb_path: &Path, ip: &str) -> Option<String> {
    let ip_addr: IpAddr = ip.parse().ok()?;
    LocalMmdbLookup::lookup_asn_label_ip(mmdb_path, ip_addr)
}

#[cfg(test)]
mod tests {
    use super::{
        GeoIpLookup, LocalMmdbLookup, lookup_asn_label, lookup_city_label, lookup_country_iso,
    };

    #[test]
    fn returns_none_for_invalid_inputs() {
        assert!(lookup_country_iso("/tmp/no-such.mmdb".as_ref(), "8.8.8.8").is_none());
        assert!(lookup_city_label("/tmp/no-such.mmdb".as_ref(), "8.8.8.8").is_none());
        assert!(lookup_asn_label("/tmp/no-such.mmdb".as_ref(), "8.8.8.8").is_none());
        assert!(lookup_country_iso("/tmp/no-such.mmdb".as_ref(), "not-ip").is_none());
    }

    #[test]
    fn backend_name_is_mmdb() {
        let lookup =
            LocalMmdbLookup::new("country.mmdb".into(), "city.mmdb".into(), "asn.mmdb".into());

        assert_eq!(lookup.backend_name(), "mmdb");
    }

    #[test]
    fn looks_up_country_from_real_mmdb_when_provided() {
        let Some(path) = std::env::var_os("XRAT_GEOIP_TEST_MMDB") else {
            return;
        };
        let Some(code) = lookup_country_iso(path.as_ref(), "8.8.8.8") else {
            panic!("expected country code from provided mmdb");
        };
        assert_eq!(code.len(), 2);
        assert!(code.chars().all(|ch| ch.is_ascii_uppercase()));
    }

    #[test]
    fn looks_up_city_from_real_mmdb_when_provided() {
        let Some(path) = std::env::var_os("XRAT_GEOIP_TEST_CITY_MMDB") else {
            return;
        };
        let Some(value) = lookup_city_label(path.as_ref(), "8.8.8.8") else {
            panic!("expected city label from provided mmdb");
        };
        assert!(!value.is_empty());
    }

    #[test]
    fn looks_up_asn_from_real_mmdb_when_provided() {
        let Some(path) = std::env::var_os("XRAT_GEOIP_TEST_ASN_MMDB") else {
            return;
        };
        let Some(value) = lookup_asn_label(path.as_ref(), "8.8.8.8") else {
            panic!("expected asn label from provided mmdb");
        };
        assert!(value.starts_with("AS"));
    }
}
