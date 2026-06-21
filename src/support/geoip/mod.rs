mod backend;
mod cache;
mod chain;
mod classify;
mod enrich;
mod fronting;
mod local;
mod rate_limit;
mod remote_ip_api;
mod remote_ipwhois;

use std::net::IpAddr;

use std::fmt::Debug;

pub use backend::build_lookup_chain;
pub use cache::CachedLookup;
pub use chain::ChainedLookup;
pub use classify::{classify_endpoint_location, is_classified_placeholder};
pub use enrich::{EndpointGeoMeta, GeoIpSource, address_host, enrich_address, resolve_address_ip};
pub use fronting::detect_fronting;
pub use local::{LocalMmdbLookup, lookup_asn_label, lookup_city_label, lookup_country_iso};
pub use rate_limit::RateLimitedLookup;
pub use remote_ip_api::RemoteIpApiLookup;
pub use remote_ipwhois::RemoteIpWhoisLookup;

#[derive(Debug, thiserror::Error)]
pub enum GeoIpError {
    #[error("geoip HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("geoip service returned status {status} for {ip}")]
    Status {
        ip: String,
        status: u16,
        body_preview: String,
    },
    #[error("geoip service response was malformed: {0}")]
    Parse(String),
    #[error("geoip rate limit exceeded; retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    #[error("geoip backend not configured")]
    NotConfigured,
}

impl From<GeoIpError> for crate::app::AppError {
    fn from(error: GeoIpError) -> Self {
        crate::app::AppError::InvalidArgument(error.to_string())
    }
}

#[async_trait::async_trait]
pub trait GeoIpLookup: Send + Sync + Debug {
    async fn country(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError>;
    async fn city(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError>;
    async fn asn(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError>;
    fn backend_name(&self) -> &'static str;
}
