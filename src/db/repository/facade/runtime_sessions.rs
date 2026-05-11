use crate::db::connection::DbPool;
use crate::db::model::{RuntimeSessionInsert, RuntimeSessionRecord, RuntimeSessionStatus};
use crate::db::repository::runtime_sessions;

pub async fn get_runtime_session_count(pool: &DbPool) -> crate::db::Result<i64> {
    runtime_sessions::get_count(pool).await
}

pub async fn insert_runtime_session(
    pool: &DbPool,
    session: &RuntimeSessionInsert,
) -> crate::db::Result<i64> {
    runtime_sessions::insert(pool, session).await
}

pub async fn get_latest_runtime_session(
    pool: &DbPool,
) -> crate::db::Result<Option<RuntimeSessionRecord>> {
    runtime_sessions::get_latest(pool).await
}

pub async fn get_running_runtime_session(
    pool: &DbPool,
) -> crate::db::Result<Option<RuntimeSessionRecord>> {
    runtime_sessions::get_running(pool).await
}

pub async fn update_runtime_session_state(
    pool: &DbPool,
    session_id: i64,
    status: RuntimeSessionStatus,
    process_id: Option<i64>,
    started_at: Option<&str>,
    stopped_at: Option<&str>,
    failure_reason: Option<&str>,
) -> crate::db::Result<()> {
    runtime_sessions::update_state(
        pool,
        session_id,
        status,
        process_id,
        started_at,
        stopped_at,
        failure_reason,
    )
    .await
}

pub async fn mark_runtime_session_stopped(
    pool: &DbPool,
    session_id: i64,
    stopped_at: Option<&str>,
) -> crate::db::Result<()> {
    runtime_sessions::mark_stopped(pool, session_id, stopped_at).await
}

pub async fn update_runtime_session_transition_metadata(
    pool: &DbPool,
    session_id: i64,
    owner_kind: Option<&str>,
    owner_instance_id: Option<&str>,
    reason_code: Option<&str>,
    reason_detail: Option<&str>,
    transition_origin: Option<&str>,
) -> crate::db::Result<()> {
    runtime_sessions::update_transition_metadata(
        pool,
        session_id,
        owner_kind,
        owner_instance_id,
        reason_code,
        reason_detail,
        transition_origin,
    )
    .await
}

pub async fn update_runtime_session_failure_tracking(
    pool: &DbPool,
    session_id: i64,
    cooldown_until: Option<&str>,
    last_failed_at: Option<&str>,
    last_failed_reason_code: Option<&str>,
) -> crate::db::Result<()> {
    runtime_sessions::update_failure_tracking(
        pool,
        session_id,
        cooldown_until,
        last_failed_at,
        last_failed_reason_code,
    )
    .await
}
