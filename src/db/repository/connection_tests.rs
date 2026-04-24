use sqlx::{Row, SqlitePool};

use crate::db::model::{ConnectionTestInsert, ConnectionTestRecord};

pub async fn get_count(pool: &SqlitePool) -> Result<i64, Box<dyn std::error::Error>> {
    Ok(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM connection_tests")
            .fetch_one(pool)
            .await?,
    )
}

pub async fn insert(
    pool: &SqlitePool,
    test: &ConnectionTestInsert,
) -> Result<i64, Box<dyn std::error::Error>> {
    let result = sqlx::query(
        "INSERT INTO connection_tests (config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, failure_kind, failure_reason)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
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
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn list_by_config(
    pool: &SqlitePool,
    config_id: i64,
) -> Result<Vec<ConnectionTestRecord>, Box<dyn std::error::Error>> {
    let rows = sqlx::query(
        "SELECT id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, failure_kind, failure_reason, tested_at
         FROM connection_tests
         WHERE config_id = ?1
         ORDER BY tested_at DESC, id DESC",
    )
    .bind(config_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(map_connection_test_row).collect())
}

pub async fn get_latest_by_config(
    pool: &SqlitePool,
    config_id: i64,
) -> Result<Option<ConnectionTestRecord>, Box<dyn std::error::Error>> {
    let row = sqlx::query(
        "SELECT id, config_id, icmp_ok, icmp_ms, tcp_ok, tcp_ms, real_delay_ok, real_delay_ms, failure_kind, failure_reason, tested_at
         FROM connection_tests
         WHERE config_id = ?1
         ORDER BY tested_at DESC, id DESC
         LIMIT 1",
    )
    .bind(config_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(map_connection_test_row))
}

fn map_connection_test_row(row: sqlx::sqlite::SqliteRow) -> ConnectionTestRecord {
    ConnectionTestRecord {
        id: row.get("id"),
        config_id: row.get("config_id"),
        icmp_ok: row.get::<Option<i64>, _>("icmp_ok").map(|value| value != 0),
        icmp_ms: row.get("icmp_ms"),
        tcp_ok: row.get::<Option<i64>, _>("tcp_ok").map(|value| value != 0),
        tcp_ms: row.get("tcp_ms"),
        real_delay_ok: row
            .get::<Option<i64>, _>("real_delay_ok")
            .map(|value| value != 0),
        real_delay_ms: row.get("real_delay_ms"),
        failure_kind: row.get("failure_kind"),
        failure_reason: row.get("failure_reason"),
        tested_at: row.get("tested_at"),
    }
}
