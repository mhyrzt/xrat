use super::row::{map_connection_test_row, map_connection_test_run_row};
use crate::db::connection::DbPool;
use crate::db::model::{
    ConnectionTestInsert, ConnectionTestRecord, ConnectionTestRunInsert, ConnectionTestRunRecord,
};

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

pub async fn insert(pool: &DbPool, test: &ConnectionTestInsert) -> crate::db::Result<i64> {
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query_scalar(
            "INSERT INTO connection_tests (run_id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, download_mbps, upload_mbps, connect_ms, ttfb_ms, http_status, endpoint_ip, endpoint_location, endpoint_country, endpoint_asn, failure_kind, failure_reason)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19) RETURNING id",
        )
        .bind(test.run_id)
        .bind(test.config_id)
        .bind(test.icmp_ok.map(i64::from))
        .bind(test.icmp_ms)
        .bind(test.tcp_ok.map(i64::from))
        .bind(test.tcp_ms)
        .bind(test.real_delay_ok.map(i64::from))
        .bind(test.real_delay_ms)
        .bind(test.download_mbps)
        .bind(test.upload_mbps)
        .bind(test.connect_ms)
        .bind(test.ttfb_ms)
        .bind(test.http_status)
        .bind(&test.endpoint_ip)
        .bind(&test.endpoint_location)
        .bind(&test.endpoint_country)
        .bind(&test.endpoint_asn)
        .bind(&test.failure_kind)
        .bind(&test.failure_reason)
        .fetch_one(pool)
        .await?),
        DbPool::Postgres(pool) => Ok(sqlx::query_scalar(
            "INSERT INTO connection_tests (run_id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, download_mbps, upload_mbps, connect_ms, ttfb_ms, http_status, endpoint_ip, endpoint_location, endpoint_country, endpoint_asn, failure_kind, failure_reason)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19) RETURNING id",
        )
        .bind(test.run_id)
        .bind(test.config_id)
        .bind(test.icmp_ok.map(i64::from))
        .bind(test.icmp_ms)
        .bind(test.tcp_ok.map(i64::from))
        .bind(test.tcp_ms)
        .bind(test.real_delay_ok.map(i64::from))
        .bind(test.real_delay_ms)
        .bind(test.download_mbps)
        .bind(test.upload_mbps)
        .bind(test.connect_ms)
        .bind(test.ttfb_ms)
        .bind(test.http_status)
        .bind(&test.endpoint_ip)
        .bind(&test.endpoint_location)
        .bind(&test.endpoint_country)
        .bind(&test.endpoint_asn)
        .bind(&test.failure_kind)
        .bind(&test.failure_reason)
        .fetch_one(pool)
        .await?),
    }
}

pub async fn insert_run(pool: &DbPool, run: &ConnectionTestRunInsert) -> crate::db::Result<i64> {
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query_scalar(
            "INSERT INTO connection_test_runs (kind) VALUES (?1) RETURNING id",
        )
        .bind(&run.kind)
        .fetch_one(pool)
        .await?),
        DbPool::Postgres(pool) => Ok(sqlx::query_scalar(
            "INSERT INTO connection_test_runs (kind) VALUES ($1) RETURNING id",
        )
        .bind(&run.kind)
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
            "SELECT id, run_id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, download_mbps, upload_mbps, connect_ms, ttfb_ms, http_status, endpoint_ip, endpoint_location, endpoint_country, endpoint_asn, failure_kind, failure_reason, tested_at FROM connection_tests WHERE config_id = ?1 ORDER BY tested_at DESC, id DESC",
        ).bind(config_id).fetch_all(pool).await?.into_iter().map(map_connection_test_row).collect()),
        DbPool::Postgres(pool) => Ok(sqlx::query(
            "SELECT id, run_id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, download_mbps, upload_mbps, connect_ms, ttfb_ms, http_status, endpoint_ip, endpoint_location, endpoint_country, endpoint_asn, failure_kind, failure_reason, tested_at FROM connection_tests WHERE config_id = $1 ORDER BY tested_at DESC, id DESC",
        ).bind(config_id).fetch_all(pool).await?.into_iter().map(map_connection_test_row).collect()),
    }
}

pub async fn get_latest_by_config(
    pool: &DbPool,
    config_id: i64,
) -> crate::db::Result<Option<ConnectionTestRecord>> {
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query(
            "SELECT id, run_id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, download_mbps, upload_mbps, connect_ms, ttfb_ms, http_status, endpoint_ip, endpoint_location, endpoint_country, endpoint_asn, failure_kind, failure_reason, tested_at FROM connection_tests WHERE config_id = ?1 ORDER BY tested_at DESC, id DESC LIMIT 1",
        ).bind(config_id).fetch_optional(pool).await?.map(map_connection_test_row)),
        DbPool::Postgres(pool) => Ok(sqlx::query(
            "SELECT id, run_id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, download_mbps, upload_mbps, connect_ms, ttfb_ms, http_status, endpoint_ip, endpoint_location, endpoint_country, endpoint_asn, failure_kind, failure_reason, tested_at FROM connection_tests WHERE config_id = $1 ORDER BY tested_at DESC, id DESC LIMIT 1",
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
            "SELECT id, run_id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, download_mbps, upload_mbps, connect_ms, ttfb_ms, http_status, endpoint_ip, endpoint_location, endpoint_country, endpoint_asn, failure_kind, failure_reason, tested_at FROM connection_tests WHERE run_id = ?1 ORDER BY tested_at DESC, id DESC",
        ).bind(run_id).fetch_all(pool).await?.into_iter().map(map_connection_test_row).collect()),
        DbPool::Postgres(pool) => Ok(sqlx::query(
            "SELECT id, run_id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, download_mbps, upload_mbps, connect_ms, ttfb_ms, http_status, endpoint_ip, endpoint_location, endpoint_country, endpoint_asn, failure_kind, failure_reason, tested_at FROM connection_tests WHERE run_id = $1 ORDER BY tested_at DESC, id DESC",
        ).bind(run_id).fetch_all(pool).await?.into_iter().map(map_connection_test_row).collect()),
    }
}
