use super::row::map_runtime_session_row;
use crate::db::connection::DbPool;
use crate::db::model::{RuntimeSessionInsert, RuntimeSessionRecord, RuntimeSessionStatus};

const RUNTIME_SESSION_COLUMNS: &str = "id, config_id, status, socks_host, socks_port, http_host, http_port, shadowsocks_host, shadowsocks_port, process_id, started_at, stopped_at, created_at, updated_at";

pub async fn get_count(pool: &DbPool) -> crate::db::Result<i64> {
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM runtime_sessions",
        )
        .fetch_one(pool)
        .await?),
        DbPool::Postgres(pool) => Ok(sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM runtime_sessions",
        )
        .fetch_one(pool)
        .await?),
    }
}

pub async fn insert(pool: &DbPool, session: &RuntimeSessionInsert) -> crate::db::Result<i64> {
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query_scalar(
            "INSERT INTO runtime_sessions (config_id, status, socks_host, socks_port, http_host, http_port, shadowsocks_host, shadowsocks_port, process_id, started_at, stopped_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11) RETURNING id",
        )
        .bind(session.config_id)
        .bind(session.status.as_str())
        .bind(&session.socks_host)
        .bind(session.socks_port)
        .bind(&session.http_host)
        .bind(session.http_port)
        .bind(&session.shadowsocks_host)
        .bind(session.shadowsocks_port)
        .bind(session.process_id)
        .bind(&session.started_at)
        .bind(&session.stopped_at)
        .fetch_one(pool)
        .await?),
        DbPool::Postgres(pool) => Ok(sqlx::query_scalar(
            "INSERT INTO runtime_sessions (config_id, status, socks_host, socks_port, http_host, http_port, shadowsocks_host, shadowsocks_port, process_id, started_at, stopped_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id",
        )
        .bind(session.config_id)
        .bind(session.status.as_str())
        .bind(&session.socks_host)
        .bind(session.socks_port)
        .bind(&session.http_host)
        .bind(session.http_port)
        .bind(&session.shadowsocks_host)
        .bind(session.shadowsocks_port)
        .bind(session.process_id)
        .bind(&session.started_at)
        .bind(&session.stopped_at)
        .fetch_one(pool)
        .await?),
    }
}

pub async fn get_latest(pool: &DbPool) -> crate::db::Result<Option<RuntimeSessionRecord>> {
    let sql = format!(
        "SELECT {RUNTIME_SESSION_COLUMNS} FROM runtime_sessions ORDER BY created_at DESC, id DESC LIMIT 1"
    );
    match pool {
        DbPool::Sqlite(pool) => sqlx::query(&sql)
            .fetch_optional(pool)
            .await?
            .map(map_runtime_session_row)
            .transpose(),
        DbPool::Postgres(pool) => sqlx::query(&sql)
            .fetch_optional(pool)
            .await?
            .map(map_runtime_session_row)
            .transpose(),
    }
}

pub async fn get_running(pool: &DbPool) -> crate::db::Result<Option<RuntimeSessionRecord>> {
    let sql = format!(
        "SELECT {RUNTIME_SESSION_COLUMNS} FROM runtime_sessions WHERE status IN ('starting', 'running', 'stopping') ORDER BY updated_at DESC, id DESC LIMIT 1"
    );
    match pool {
        DbPool::Sqlite(pool) => sqlx::query(&sql)
            .fetch_optional(pool)
            .await?
            .map(map_runtime_session_row)
            .transpose(),
        DbPool::Postgres(pool) => sqlx::query(&sql)
            .fetch_optional(pool)
            .await?
            .map(map_runtime_session_row)
            .transpose(),
    }
}

pub async fn update_state(
    pool: &DbPool,
    session_id: i64,
    status: RuntimeSessionStatus,
    process_id: Option<i64>,
    started_at: Option<&str>,
    stopped_at: Option<&str>,
) -> crate::db::Result<()> {
    match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query("UPDATE runtime_sessions SET status = ?2, process_id = COALESCE(?3, process_id), started_at = COALESCE(?4, started_at), stopped_at = COALESCE(?5, stopped_at), updated_at = CURRENT_TIMESTAMP WHERE id = ?1")
                .bind(session_id).bind(status.as_str()).bind(process_id).bind(started_at).bind(stopped_at).execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query("UPDATE runtime_sessions SET status = $2, process_id = COALESCE($3, process_id), started_at = COALESCE($4, started_at), stopped_at = COALESCE($5, stopped_at), updated_at = CURRENT_TIMESTAMP::TEXT WHERE id = $1")
                .bind(session_id).bind(status.as_str()).bind(process_id).bind(started_at).bind(stopped_at).execute(pool).await?;
        }
    }
    Ok(())
}

pub async fn mark_stopped(
    pool: &DbPool,
    session_id: i64,
    stopped_at: Option<&str>,
) -> crate::db::Result<()> {
    match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query("UPDATE runtime_sessions SET status = 'stopped', stopped_at = COALESCE(?2, stopped_at, CURRENT_TIMESTAMP), updated_at = CURRENT_TIMESTAMP WHERE id = ?1")
                .bind(session_id).bind(stopped_at).execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query("UPDATE runtime_sessions SET status = 'stopped', stopped_at = COALESCE($2, stopped_at, CURRENT_TIMESTAMP::TEXT), updated_at = CURRENT_TIMESTAMP::TEXT WHERE id = $1")
                .bind(session_id).bind(stopped_at).execute(pool).await?;
        }
    }
    Ok(())
}
