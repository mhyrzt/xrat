use super::row::map_runtime_session_row;
use crate::db::connection::DbPool;
use crate::db::model::{RuntimeSessionInsert, RuntimeSessionRecord, RuntimeSessionStatus};

const RUNTIME_SESSION_COLUMNS: &str = "id, config_id, status, socks_host, socks_port, http_host, http_port, shadowsocks_host, shadowsocks_port, process_id, failure_reason, owner_kind, owner_instance_id, last_transition_reason_code, last_transition_reason_detail, last_transition_origin, cooldown_until, last_failed_at, last_failed_reason_code, started_at, stopped_at, created_at, updated_at";

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
            "INSERT INTO runtime_sessions (config_id, status, socks_host, socks_port, http_host, http_port, shadowsocks_host, shadowsocks_port, process_id, failure_reason, started_at, stopped_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12) RETURNING id",
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
        .bind(&session.failure_reason)
        .bind(&session.started_at)
        .bind(&session.stopped_at)
        .fetch_one(pool)
        .await?),
        DbPool::Postgres(pool) => Ok(sqlx::query_scalar(
            "INSERT INTO runtime_sessions (config_id, status, socks_host, socks_port, http_host, http_port, shadowsocks_host, shadowsocks_port, process_id, failure_reason, started_at, stopped_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING id",
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
        .bind(&session.failure_reason)
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

pub async fn get_latest_for_config(
    pool: &DbPool,
    config_id: i64,
) -> crate::db::Result<Option<RuntimeSessionRecord>> {
    let sql = format!(
        "SELECT {RUNTIME_SESSION_COLUMNS} FROM runtime_sessions WHERE config_id = ?1 ORDER BY created_at DESC, id DESC LIMIT 1"
    );
    match pool {
        DbPool::Sqlite(pool) => sqlx::query(&sql)
            .bind(config_id)
            .fetch_optional(pool)
            .await?
            .map(map_runtime_session_row)
            .transpose(),
        DbPool::Postgres(pool) => {
            let sql = format!(
                "SELECT {RUNTIME_SESSION_COLUMNS} FROM runtime_sessions WHERE config_id = $1 ORDER BY created_at DESC, id DESC LIMIT 1"
            );
            sqlx::query(&sql)
                .bind(config_id)
                .fetch_optional(pool)
                .await?
                .map(map_runtime_session_row)
                .transpose()
        }
    }
}

pub async fn update_state(
    pool: &DbPool,
    session_id: i64,
    status: RuntimeSessionStatus,
    process_id: Option<i64>,
    started_at: Option<&str>,
    stopped_at: Option<&str>,
    failure_reason: Option<&str>,
) -> crate::db::Result<()> {
    match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query("UPDATE runtime_sessions SET status = ?2, process_id = COALESCE(?3, process_id), started_at = COALESCE(?4, started_at), stopped_at = COALESCE(?5, stopped_at), failure_reason = COALESCE(?6, failure_reason), updated_at = CURRENT_TIMESTAMP WHERE id = ?1")
                .bind(session_id).bind(status.as_str()).bind(process_id).bind(started_at).bind(stopped_at).bind(failure_reason).execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query("UPDATE runtime_sessions SET status = $2, process_id = COALESCE($3, process_id), started_at = COALESCE($4, started_at), stopped_at = COALESCE($5, stopped_at), failure_reason = COALESCE($6, failure_reason), updated_at = CURRENT_TIMESTAMP::TEXT WHERE id = $1")
                .bind(session_id).bind(status.as_str()).bind(process_id).bind(started_at).bind(stopped_at).bind(failure_reason).execute(pool).await?;
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

pub async fn update_transition_metadata(
    pool: &DbPool,
    session_id: i64,
    owner_kind: Option<&str>,
    owner_instance_id: Option<&str>,
    reason_code: Option<&str>,
    reason_detail: Option<&str>,
    transition_origin: Option<&str>,
) -> crate::db::Result<()> {
    match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query("UPDATE runtime_sessions SET owner_kind = COALESCE(?2, owner_kind), owner_instance_id = COALESCE(?3, owner_instance_id), last_transition_reason_code = COALESCE(?4, last_transition_reason_code), last_transition_reason_detail = COALESCE(?5, last_transition_reason_detail), last_transition_origin = COALESCE(?6, last_transition_origin), updated_at = CURRENT_TIMESTAMP WHERE id = ?1")
                .bind(session_id)
                .bind(owner_kind)
                .bind(owner_instance_id)
                .bind(reason_code)
                .bind(reason_detail)
                .bind(transition_origin)
                .execute(pool)
                .await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query("UPDATE runtime_sessions SET owner_kind = COALESCE($2, owner_kind), owner_instance_id = COALESCE($3, owner_instance_id), last_transition_reason_code = COALESCE($4, last_transition_reason_code), last_transition_reason_detail = COALESCE($5, last_transition_reason_detail), last_transition_origin = COALESCE($6, last_transition_origin), updated_at = CURRENT_TIMESTAMP::TEXT WHERE id = $1")
                .bind(session_id)
                .bind(owner_kind)
                .bind(owner_instance_id)
                .bind(reason_code)
                .bind(reason_detail)
                .bind(transition_origin)
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}

pub async fn update_failure_tracking(
    pool: &DbPool,
    session_id: i64,
    cooldown_until: Option<&str>,
    last_failed_at: Option<&str>,
    last_failed_reason_code: Option<&str>,
) -> crate::db::Result<()> {
    match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query("UPDATE runtime_sessions SET cooldown_until = COALESCE(?2, cooldown_until), last_failed_at = COALESCE(?3, last_failed_at), last_failed_reason_code = COALESCE(?4, last_failed_reason_code), updated_at = CURRENT_TIMESTAMP WHERE id = ?1")
                .bind(session_id)
                .bind(cooldown_until)
                .bind(last_failed_at)
                .bind(last_failed_reason_code)
                .execute(pool)
                .await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query("UPDATE runtime_sessions SET cooldown_until = COALESCE($2, cooldown_until), last_failed_at = COALESCE($3, last_failed_at), last_failed_reason_code = COALESCE($4, last_failed_reason_code), updated_at = CURRENT_TIMESTAMP::TEXT WHERE id = $1")
                .bind(session_id)
                .bind(cooldown_until)
                .bind(last_failed_at)
                .bind(last_failed_reason_code)
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}
