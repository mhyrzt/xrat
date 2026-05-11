use super::Database;
use super::imports::*;

impl Database {
    pub async fn get_runtime_session_count(&self) -> crate::db::Result<i64> {
        repository::get_runtime_session_count(&self.pool).await
    }

    pub async fn insert_runtime_session(
        &self,
        session: &RuntimeSessionInsert,
    ) -> crate::db::Result<i64> {
        repository::insert_runtime_session(&self.pool, session).await
    }

    pub async fn get_latest_runtime_session(
        &self,
    ) -> crate::db::Result<Option<RuntimeSessionRecord>> {
        repository::get_latest_runtime_session(&self.pool).await
    }

    pub async fn get_running_runtime_session(
        &self,
    ) -> crate::db::Result<Option<RuntimeSessionRecord>> {
        repository::get_running_runtime_session(&self.pool).await
    }

    pub async fn update_runtime_session_state(
        &self,
        session_id: i64,
        status: RuntimeSessionStatus,
        process_id: Option<i64>,
        started_at: Option<&str>,
        stopped_at: Option<&str>,
        failure_reason: Option<&str>,
    ) -> crate::db::Result<()> {
        repository::update_runtime_session_state(
            &self.pool,
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
        &self,
        session_id: i64,
        stopped_at: Option<&str>,
    ) -> crate::db::Result<()> {
        repository::mark_runtime_session_stopped(&self.pool, session_id, stopped_at).await
    }

    pub async fn update_runtime_session_transition_metadata(
        &self,
        session_id: i64,
        owner_kind: Option<&str>,
        owner_instance_id: Option<&str>,
        reason_code: Option<&str>,
        reason_detail: Option<&str>,
        transition_origin: Option<&str>,
    ) -> crate::db::Result<()> {
        repository::update_runtime_session_transition_metadata(
            &self.pool,
            session_id,
            owner_kind,
            owner_instance_id,
            reason_code,
            reason_detail,
            transition_origin,
        )
        .await
    }
}
