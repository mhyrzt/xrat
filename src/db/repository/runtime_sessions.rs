use sqlx::{Row, SqlitePool};

use crate::db::model::{RuntimeSessionInsert, RuntimeSessionRecord, RuntimeSessionStatus};

pub async fn get_count(pool: &SqlitePool) -> Result<i64, Box<dyn std::error::Error>> {
    Ok(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM runtime_sessions")
            .fetch_one(pool)
            .await?,
    )
}

pub async fn insert(
    pool: &SqlitePool,
    session: &RuntimeSessionInsert,
) -> Result<i64, Box<dyn std::error::Error>> {
    let result = sqlx::query(
        "INSERT INTO runtime_sessions (config_id, status, mixed_port, process_id, started_at, stopped_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(session.config_id)
    .bind(session.status.as_str())
    .bind(session.mixed_port)
    .bind(session.process_id)
    .bind(&session.started_at)
    .bind(&session.stopped_at)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn get_latest(
    pool: &SqlitePool,
) -> Result<Option<RuntimeSessionRecord>, Box<dyn std::error::Error>> {
    let row = sqlx::query(
        "SELECT id, config_id, status, mixed_port, process_id, started_at, stopped_at, created_at, updated_at
         FROM runtime_sessions
         ORDER BY created_at DESC, id DESC
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    row.map(map_runtime_session_row).transpose()
}

pub async fn get_running(
    pool: &SqlitePool,
) -> Result<Option<RuntimeSessionRecord>, Box<dyn std::error::Error>> {
    let row = sqlx::query(
        "SELECT id, config_id, status, mixed_port, process_id, started_at, stopped_at, created_at, updated_at
         FROM runtime_sessions
         WHERE status IN ('starting', 'running', 'stopping')
         ORDER BY updated_at DESC, id DESC
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    row.map(map_runtime_session_row).transpose()
}

pub async fn update_state(
    pool: &SqlitePool,
    session_id: i64,
    status: RuntimeSessionStatus,
    process_id: Option<i64>,
    mixed_port: Option<i64>,
    started_at: Option<&str>,
    stopped_at: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query(
        "UPDATE runtime_sessions
         SET status = ?2,
             process_id = COALESCE(?3, process_id),
             mixed_port = COALESCE(?4, mixed_port),
             started_at = COALESCE(?5, started_at),
             stopped_at = COALESCE(?6, stopped_at),
             updated_at = CURRENT_TIMESTAMP
         WHERE id = ?1",
    )
    .bind(session_id)
    .bind(status.as_str())
    .bind(process_id)
    .bind(mixed_port)
    .bind(started_at)
    .bind(stopped_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_stopped(
    pool: &SqlitePool,
    session_id: i64,
    stopped_at: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query(
        "UPDATE runtime_sessions
         SET status = 'stopped',
             stopped_at = COALESCE(?2, stopped_at, CURRENT_TIMESTAMP),
             updated_at = CURRENT_TIMESTAMP
         WHERE id = ?1",
    )
    .bind(session_id)
    .bind(stopped_at)
    .execute(pool)
    .await?;

    Ok(())
}

fn map_runtime_session_row(
    row: sqlx::sqlite::SqliteRow,
) -> Result<RuntimeSessionRecord, Box<dyn std::error::Error>> {
    let status_value: String = row.get("status");
    let status = RuntimeSessionStatus::from_str(&status_value)
        .ok_or_else(|| format!("invalid runtime session status: {status_value}"))?;

    Ok(RuntimeSessionRecord {
        id: row.get("id"),
        config_id: row.get("config_id"),
        status,
        mixed_port: row.get("mixed_port"),
        process_id: row.get("process_id"),
        started_at: row.get("started_at"),
        stopped_at: row.get("stopped_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
