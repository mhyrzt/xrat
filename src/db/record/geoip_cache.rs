/// A resolved host -> geo mapping, persisted so endpoint locations survive
/// restarts and unresolvable hosts stop re-resolving on every launch. Empty
/// geo fields are cached too: they record a recent resolution attempt so a dead
/// host is not retried until the entry goes stale.
#[derive(Clone, Debug, PartialEq)]
pub struct GeoIpCacheRecord {
    pub host: String,
    pub ip: Option<String>,
    pub country: Option<String>,
    pub location: Option<String>,
    pub asn: Option<String>,
    /// Unix epoch seconds at which this entry was resolved.
    pub resolved_at: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeoIpCacheUpsert {
    pub host: String,
    pub ip: Option<String>,
    pub country: Option<String>,
    pub location: Option<String>,
    pub asn: Option<String>,
    pub resolved_at: i64,
}

impl GeoIpCacheRecord {
    pub fn has_location(&self) -> bool {
        self.country.is_some() || self.location.is_some() || self.asn.is_some()
    }
}
