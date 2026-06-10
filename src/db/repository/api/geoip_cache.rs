use crate::db::connection::DbPool;
use crate::db::record::{GeoIpCacheRecord, GeoIpCacheUpsert};
use crate::db::repository::geoip_cache;

pub async fn get_fresh_geoip_cache(
    pool: &DbPool,
    hosts: &[String],
    min_resolved_at: i64,
) -> crate::db::Result<Vec<GeoIpCacheRecord>> {
    geoip_cache::get_fresh(pool, hosts, min_resolved_at).await
}

pub async fn upsert_geoip_cache(pool: &DbPool, entry: &GeoIpCacheUpsert) -> crate::db::Result<()> {
    geoip_cache::upsert(pool, entry).await
}
