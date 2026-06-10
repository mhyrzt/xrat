use super::Database;
use super::types::*;

impl Database {
    /// Cached geo for `hosts` resolved at or after `min_resolved_at` (unix
    /// epoch seconds). Stale or missing hosts are simply absent from the result.
    pub async fn get_fresh_geoip_cache(
        &self,
        hosts: &[String],
        min_resolved_at: i64,
    ) -> crate::db::Result<Vec<GeoIpCacheRecord>> {
        repository::get_fresh_geoip_cache(&self.pool, hosts, min_resolved_at).await
    }

    pub async fn upsert_geoip_cache(&self, entry: &GeoIpCacheUpsert) -> crate::db::Result<()> {
        repository::upsert_geoip_cache(&self.pool, entry).await
    }
}
