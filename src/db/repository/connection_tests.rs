use super::row::map_connection_test_row;
use crate::db::connection::DbPool;
use crate::db::model::{ConnectionTestInsert, ConnectionTestRecord};

pub async fn get_count(pool: &DbPool) -> Result<i64, Box<dyn std::error::Error>> {
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

pub async fn insert(
    pool: &DbPool,
    test: &ConnectionTestInsert,
) -> Result<i64, Box<dyn std::error::Error>> {
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query_scalar(
            "INSERT INTO connection_tests (config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, failure_kind, failure_reason)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) RETURNING id",
        )
        .bind(test.config_id)
        .bind(test.icmp_ok.map(i64::from))
        .bind(test.icmp_ms)
        .bind(test.tcp_ok.map(i64::from))
        .bind(test.tcp_ms)
        .bind(test.real_delay_ok.map(i64::from))
        .bind(test.real_delay_ms)
        .bind(&test.failure_kind)
        .bind(&test.failure_reason)
        .fetch_one(pool)
        .await?),
        DbPool::Postgres(pool) => Ok(sqlx::query_scalar(
            "INSERT INTO connection_tests (config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, failure_kind, failure_reason)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id",
        )
        .bind(test.config_id)
        .bind(test.icmp_ok.map(i64::from))
        .bind(test.icmp_ms)
        .bind(test.tcp_ok.map(i64::from))
        .bind(test.tcp_ms)
        .bind(test.real_delay_ok.map(i64::from))
        .bind(test.real_delay_ms)
        .bind(&test.failure_kind)
        .bind(&test.failure_reason)
        .fetch_one(pool)
        .await?),
    }
}

pub async fn list_by_config(
    pool: &DbPool,
    config_id: i64,
) -> Result<Vec<ConnectionTestRecord>, Box<dyn std::error::Error>> {
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query(
            "SELECT id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, failure_kind, failure_reason, tested_at FROM connection_tests WHERE config_id = ?1 ORDER BY tested_at DESC, id DESC",
        ).bind(config_id).fetch_all(pool).await?.into_iter().map(map_connection_test_row).collect()),
        DbPool::Postgres(pool) => Ok(sqlx::query(
            "SELECT id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, failure_kind, failure_reason, tested_at FROM connection_tests WHERE config_id = $1 ORDER BY tested_at DESC, id DESC",
        ).bind(config_id).fetch_all(pool).await?.into_iter().map(map_connection_test_row).collect()),
    }
}

pub async fn get_latest_by_config(
    pool: &DbPool,
    config_id: i64,
) -> Result<Option<ConnectionTestRecord>, Box<dyn std::error::Error>> {
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query(
            "SELECT id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, failure_kind, failure_reason, tested_at FROM connection_tests WHERE config_id = ?1 ORDER BY tested_at DESC, id DESC LIMIT 1",
        ).bind(config_id).fetch_optional(pool).await?.map(map_connection_test_row)),
        DbPool::Postgres(pool) => Ok(sqlx::query(
            "SELECT id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, failure_kind, failure_reason, tested_at FROM connection_tests WHERE config_id = $1 ORDER BY tested_at DESC, id DESC LIMIT 1",
        ).bind(config_id).fetch_optional(pool).await?.map(map_connection_test_row)),
    }
}
