use crate::db::connection::DbPool;
use crate::db::model::{ConnectionTestInsert, ConnectionTestRunInsert};

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
