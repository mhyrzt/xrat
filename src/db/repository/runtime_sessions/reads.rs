use super::*;

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

pub async fn get_latest(pool: &DbPool) -> crate::db::Result<Option<RuntimeSessionRecord>> {
    let sql = format!(
        "SELECT {RUNTIME_SESSION_COLUMNS} FROM runtime_sessions ORDER BY created_at DESC, id DESC LIMIT 1"
    );
    fetch_optional_runtime_session(pool, &sql, &sql, None).await
}

pub async fn get_running(pool: &DbPool) -> crate::db::Result<Option<RuntimeSessionRecord>> {
    let sql = format!(
        "SELECT {RUNTIME_SESSION_COLUMNS} FROM runtime_sessions WHERE status IN ('starting', 'running', 'stopping') ORDER BY updated_at DESC, id DESC LIMIT 1"
    );
    fetch_optional_runtime_session(pool, &sql, &sql, None).await
}

pub async fn get_latest_for_config(
    pool: &DbPool,
    config_id: i64,
) -> crate::db::Result<Option<RuntimeSessionRecord>> {
    let sqlite_sql = format!(
        "SELECT {RUNTIME_SESSION_COLUMNS} FROM runtime_sessions WHERE config_id = ?1 ORDER BY created_at DESC, id DESC LIMIT 1"
    );
    let postgres_sql = format!(
        "SELECT {RUNTIME_SESSION_COLUMNS} FROM runtime_sessions WHERE config_id = $1 ORDER BY created_at DESC, id DESC LIMIT 1"
    );
    fetch_optional_runtime_session(pool, &sqlite_sql, &postgres_sql, Some(config_id)).await
}
