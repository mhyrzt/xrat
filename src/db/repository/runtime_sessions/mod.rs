use super::row::map_runtime_session_row;
use crate::db::connection::DbPool;
use crate::db::model::RuntimeSessionRecord;

mod reads;
mod writes;

const RUNTIME_SESSION_COLUMNS: &str = "id, config_id, status, socks_host, socks_port, http_host, http_port, shadowsocks_host, shadowsocks_port, process_id, failure_reason, owner_kind, owner_instance_id, last_transition_reason_code, last_transition_reason_detail, last_transition_origin, cooldown_until, last_failed_at, last_failed_reason_code, started_at, stopped_at, created_at, updated_at";

pub use reads::{get_count, get_latest, get_latest_for_config, get_running};
pub use writes::{
    insert, mark_stopped, update_failure_tracking, update_state, update_transition_metadata,
};

async fn fetch_optional_runtime_session(
    pool: &DbPool,
    sqlite_sql: &str,
    postgres_sql: &str,
    bind_config_id: Option<i64>,
) -> crate::db::Result<Option<RuntimeSessionRecord>> {
    match pool {
        DbPool::Sqlite(pool) => {
            let mut query = sqlx::query(sqlite_sql);
            if let Some(config_id) = bind_config_id {
                query = query.bind(config_id);
            }
            query
                .fetch_optional(pool)
                .await?
                .map(map_runtime_session_row)
                .transpose()
        }
        DbPool::Postgres(pool) => {
            let mut query = sqlx::query(postgres_sql);
            if let Some(config_id) = bind_config_id {
                query = query.bind(config_id);
            }
            query
                .fetch_optional(pool)
                .await?
                .map(map_runtime_session_row)
                .transpose()
        }
    }
}
