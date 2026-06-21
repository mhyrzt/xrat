use super::super::row::{map_connection_test_row, map_connection_test_run_row};
use crate::db::connection::DbPool;
use crate::db::record::{ConnectionTestRecord, ConnectionTestRunRecord};

pub async fn get_count(pool: &DbPool) -> crate::db::Result<i64> {
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM connection_tests",
        )
        .fetch_one(pool)
        .await?),
        DbPool::Postgres(pool) => Ok(sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM connection_tests",
        )
        .fetch_one(pool)
        .await?),
    }
}

pub async fn list_by_config(
    pool: &DbPool,
    config_id: i64,
) -> crate::db::Result<Vec<ConnectionTestRecord>> {
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query(
            "SELECT id, run_id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, download_mbps, upload_mbps, connect_ms, ttfb_ms, http_status, dial_endpoint_ip, dial_endpoint_location, dial_endpoint_country, dial_endpoint_asn, dial_endpoint_geoip_source, dial_endpoint_fronting, failure_kind, failure_reason, tested_at FROM connection_tests WHERE config_id = ?1 ORDER BY tested_at DESC, id DESC",
        ).bind(config_id).fetch_all(pool).await?.into_iter().map(map_connection_test_row).collect()),
        DbPool::Postgres(pool) => Ok(sqlx::query(
            "SELECT id, run_id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, download_mbps, upload_mbps, connect_ms, ttfb_ms, http_status, dial_endpoint_ip, dial_endpoint_location, dial_endpoint_country, dial_endpoint_asn, dial_endpoint_geoip_source, dial_endpoint_fronting, failure_kind, failure_reason, tested_at FROM connection_tests WHERE config_id = $1 ORDER BY tested_at DESC, id DESC",
        ).bind(config_id).fetch_all(pool).await?.into_iter().map(map_connection_test_row).collect()),
    }
}

pub async fn get_latest_by_config(
    pool: &DbPool,
    config_id: i64,
) -> crate::db::Result<Option<ConnectionTestRecord>> {
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query(
            "SELECT id, run_id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, download_mbps, upload_mbps, connect_ms, ttfb_ms, http_status, dial_endpoint_ip, dial_endpoint_location, dial_endpoint_country, dial_endpoint_asn, dial_endpoint_geoip_source, dial_endpoint_fronting, failure_kind, failure_reason, tested_at FROM connection_tests WHERE config_id = ?1 ORDER BY tested_at DESC, id DESC LIMIT 1",
        ).bind(config_id).fetch_optional(pool).await?.map(map_connection_test_row)),
        DbPool::Postgres(pool) => Ok(sqlx::query(
            "SELECT id, run_id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, download_mbps, upload_mbps, connect_ms, ttfb_ms, http_status, dial_endpoint_ip, dial_endpoint_location, dial_endpoint_country, dial_endpoint_asn, dial_endpoint_geoip_source, dial_endpoint_fronting, failure_kind, failure_reason, tested_at FROM connection_tests WHERE config_id = $1 ORDER BY tested_at DESC, id DESC LIMIT 1",
        ).bind(config_id).fetch_optional(pool).await?.map(map_connection_test_row)),
    }
}

pub async fn get_latest_run(pool: &DbPool) -> crate::db::Result<Option<ConnectionTestRunRecord>> {
    match pool {
        DbPool::Sqlite(pool) => Ok(
            sqlx::query("SELECT id, kind, created_at FROM connection_test_runs ORDER BY created_at DESC, id DESC LIMIT 1")
                .fetch_optional(pool)
                .await?
                .map(map_connection_test_run_row),
        ),
        DbPool::Postgres(pool) => Ok(
            sqlx::query("SELECT id, kind, created_at FROM connection_test_runs ORDER BY created_at DESC, id DESC LIMIT 1")
                .fetch_optional(pool)
                .await?
                .map(map_connection_test_run_row),
        ),
    }
}

pub async fn list_by_run(
    pool: &DbPool,
    run_id: i64,
) -> crate::db::Result<Vec<ConnectionTestRecord>> {
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query(
            "SELECT id, run_id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, download_mbps, upload_mbps, connect_ms, ttfb_ms, http_status, dial_endpoint_ip, dial_endpoint_location, dial_endpoint_country, dial_endpoint_asn, dial_endpoint_geoip_source, dial_endpoint_fronting, failure_kind, failure_reason, tested_at FROM connection_tests WHERE run_id = ?1 ORDER BY tested_at DESC, id DESC",
        ).bind(run_id).fetch_all(pool).await?.into_iter().map(map_connection_test_row).collect()),
        DbPool::Postgres(pool) => Ok(sqlx::query(
            "SELECT id, run_id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, download_mbps, upload_mbps, connect_ms, ttfb_ms, http_status, dial_endpoint_ip, dial_endpoint_location, dial_endpoint_country, dial_endpoint_asn, dial_endpoint_geoip_source, dial_endpoint_fronting, failure_kind, failure_reason, tested_at FROM connection_tests WHERE run_id = $1 ORDER BY tested_at DESC, id DESC",
        ).bind(run_id).fetch_all(pool).await?.into_iter().map(map_connection_test_row).collect()),
    }
}
