use std::net::IpAddr;
use std::path::{Path, PathBuf};

use maxminddb::{Reader, geoip2};

use super::{GeoIpError, GeoIpLookup};

#[cfg(test)]
mod tests;

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
        let country: geoip2::Country<'_> = reader.lookup(ip_addr).ok()?.decode().ok()??;

        country
            .country
            .iso_code
            .map(|code| code.to_string())
            .or_else(|| {
                country
                    .registered_country
                    .iso_code
                    .map(|code| code.to_string())
            })
    }

    fn lookup_city_label_ip(mmdb_path: &Path, ip_addr: IpAddr) -> Option<String> {
        let reader = Reader::open_readfile(mmdb_path).ok()?;
        let city: geoip2::City<'_> = reader.lookup(ip_addr).ok()?.decode().ok()??;

        let country = city.country.iso_code.map(str::to_string);
        let region = city
            .subdivisions
            .first()
            .and_then(|sub| sub.names.english)
            .map(str::to_string);
        let city_name = city.city.names.english.map(str::to_string);

        match (country, region, city_name) {
            (Some(cc), Some(region), Some(city)) => Some(format!("{cc}/{region}/{city}")),
            (Some(cc), _, Some(city)) => Some(format!("{cc}/{city}")),
            (Some(cc), _, _) => Some(cc),
            _ => None,
        }
    }

    fn lookup_asn_label_ip(mmdb_path: &Path, ip_addr: IpAddr) -> Option<String> {
        let reader = Reader::open_readfile(mmdb_path).ok()?;
        let asn: geoip2::Asn<'_> = reader.lookup(ip_addr).ok()?.decode().ok()??;
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
