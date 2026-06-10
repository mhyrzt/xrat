use sqlx::{Postgres, QueryBuilder, Sqlite};

use super::row::map_geoip_cache_row;
use crate::db::connection::DbPool;
use crate::db::record::{GeoIpCacheRecord, GeoIpCacheUpsert};

const BASE_SELECT: &str = "SELECT host, ip, country, location, asn, resolved_at FROM geoip_cache";

/// Fetch cache entries for `hosts` that were resolved at or after
/// `min_resolved_at` (unix epoch seconds). Stale entries are treated as misses.
pub async fn get_fresh(
    pool: &DbPool,
    hosts: &[String],
    min_resolved_at: i64,
) -> crate::db::Result<Vec<GeoIpCacheRecord>> {
    if hosts.is_empty() {
        return Ok(Vec::new());
    }

    match pool {
        DbPool::Sqlite(pool) => {
            let mut builder = QueryBuilder::<Sqlite>::new(BASE_SELECT);
            push_fresh_filter(&mut builder, hosts, min_resolved_at);
            Ok(builder
                .build()
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(map_geoip_cache_row)
                .collect())
        }
        DbPool::Postgres(pool) => {
            let mut builder = QueryBuilder::<Postgres>::new(BASE_SELECT);
            push_fresh_filter(&mut builder, hosts, min_resolved_at);
            Ok(builder
                .build()
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(map_geoip_cache_row)
                .collect())
        }
    }
}

/// Insert or replace a host's cached geo, refreshing `resolved_at`.
pub async fn upsert(pool: &DbPool, entry: &GeoIpCacheUpsert) -> crate::db::Result<()> {
    match pool {
        DbPool::Sqlite(pool) => {
            let mut builder = QueryBuilder::<Sqlite>::new(INSERT_PREFIX);
            push_upsert_values(&mut builder, entry);
            builder.push(UPSERT_SUFFIX);
            builder.build().execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            let mut builder = QueryBuilder::<Postgres>::new(INSERT_PREFIX);
            push_upsert_values(&mut builder, entry);
            builder.push(UPSERT_SUFFIX);
            builder.build().execute(pool).await?;
        }
    }
    Ok(())
}

const INSERT_PREFIX: &str =
    "INSERT INTO geoip_cache (host, ip, country, location, asn, resolved_at) VALUES (";

const UPSERT_SUFFIX: &str = ") ON CONFLICT(host) DO UPDATE SET ip = excluded.ip, \
country = excluded.country, location = excluded.location, asn = excluded.asn, \
resolved_at = excluded.resolved_at";

fn push_fresh_filter<'args, DB>(
    builder: &mut QueryBuilder<'args, DB>,
    hosts: &'args [String],
    min_resolved_at: i64,
) where
    DB: sqlx::Database,
    i64: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    &'args str: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    builder.push(" WHERE resolved_at >= ");
    builder.push_bind(min_resolved_at);
    builder.push(" AND host IN (");
    let mut separated = builder.separated(", ");
    for host in hosts {
        separated.push_bind(host.as_str());
    }
    separated.push_unseparated(")");
}

fn push_upsert_values<'args, DB>(
    builder: &mut QueryBuilder<'args, DB>,
    entry: &'args GeoIpCacheUpsert,
) where
    DB: sqlx::Database,
    i64: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    Option<&'args str>: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    &'args str: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    builder
        .push_bind(entry.host.as_str())
        .push(", ")
        .push_bind(entry.ip.as_deref())
        .push(", ")
        .push_bind(entry.country.as_deref())
        .push(", ")
        .push_bind(entry.location.as_deref())
        .push(", ")
        .push_bind(entry.asn.as_deref())
        .push(", ")
        .push_bind(entry.resolved_at);
}
