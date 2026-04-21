mod configs;
mod connection_tests;
mod runtime_sessions;
mod subscriptions;

use sqlx::SqlitePool;

use crate::db::model::{
    ConfigListFilter, ConfigRecord, ConnectionTestInsert, ConnectionTestRecord, ImportSource,
    ImportSummary, RuntimeSessionInsert, RuntimeSessionRecord, RuntimeSessionStatus,
    SubscriptionRecord,
};
use crate::model::Node;

pub async fn import_nodes(
    pool: &SqlitePool,
    source: &ImportSource,
    nodes: &[Node],
) -> Result<ImportSummary, Box<dyn std::error::Error>> {
    let subscription_id = subscriptions::insert(pool, source).await?;
    configs::import_nodes(pool, subscription_id, nodes).await
}

pub async fn get_config_count(pool: &SqlitePool) -> Result<i64, Box<dyn std::error::Error>> {
    configs::get_count(pool).await
}

pub async fn list_configs(
    pool: &SqlitePool,
    filter: &ConfigListFilter,
) -> Result<Vec<ConfigRecord>, Box<dyn std::error::Error>> {
    configs::list(pool, filter).await
}

pub async fn get_config_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<ConfigRecord>, Box<dyn std::error::Error>> {
    configs::get_by_id(pool, id).await
}

pub async fn get_selected_config(
    pool: &SqlitePool,
) -> Result<Option<ConfigRecord>, Box<dyn std::error::Error>> {
    configs::get_selected(pool).await
}

pub async fn get_active_config(
    pool: &SqlitePool,
) -> Result<Option<ConfigRecord>, Box<dyn std::error::Error>> {
    configs::get_active(pool).await
}

pub async fn get_subscription_count(pool: &SqlitePool) -> Result<i64, Box<dyn std::error::Error>> {
    subscriptions::get_count(pool).await
}

pub async fn list_subscriptions(
    pool: &SqlitePool,
) -> Result<Vec<SubscriptionRecord>, Box<dyn std::error::Error>> {
    subscriptions::list(pool).await
}

pub async fn get_connection_test_count(
    pool: &SqlitePool,
) -> Result<i64, Box<dyn std::error::Error>> {
    connection_tests::get_count(pool).await
}

pub async fn insert_connection_test(
    pool: &SqlitePool,
    test: &ConnectionTestInsert,
) -> Result<i64, Box<dyn std::error::Error>> {
    connection_tests::insert(pool, test).await
}

pub async fn list_connection_tests(
    pool: &SqlitePool,
    config_id: i64,
) -> Result<Vec<ConnectionTestRecord>, Box<dyn std::error::Error>> {
    connection_tests::list_by_config(pool, config_id).await
}

pub async fn get_latest_connection_test(
    pool: &SqlitePool,
    config_id: i64,
) -> Result<Option<ConnectionTestRecord>, Box<dyn std::error::Error>> {
    connection_tests::get_latest_by_config(pool, config_id).await
}

pub async fn get_runtime_session_count(
    pool: &SqlitePool,
) -> Result<i64, Box<dyn std::error::Error>> {
    runtime_sessions::get_count(pool).await
}

pub async fn insert_runtime_session(
    pool: &SqlitePool,
    session: &RuntimeSessionInsert,
) -> Result<i64, Box<dyn std::error::Error>> {
    runtime_sessions::insert(pool, session).await
}

pub async fn get_latest_runtime_session(
    pool: &SqlitePool,
) -> Result<Option<RuntimeSessionRecord>, Box<dyn std::error::Error>> {
    runtime_sessions::get_latest(pool).await
}

pub async fn get_running_runtime_session(
    pool: &SqlitePool,
) -> Result<Option<RuntimeSessionRecord>, Box<dyn std::error::Error>> {
    runtime_sessions::get_running(pool).await
}

pub async fn update_runtime_session_state(
    pool: &SqlitePool,
    session_id: i64,
    status: RuntimeSessionStatus,
    process_id: Option<i64>,
    mixed_port: Option<i64>,
    started_at: Option<&str>,
    stopped_at: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    runtime_sessions::update_state(
        pool, session_id, status, process_id, mixed_port, started_at, stopped_at,
    )
    .await
}

pub async fn mark_runtime_session_stopped(
    pool: &SqlitePool,
    session_id: i64,
    stopped_at: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    runtime_sessions::mark_stopped(pool, session_id, stopped_at).await
}

pub async fn get_config_flags(
    pool: &SqlitePool,
    dedup_key: &str,
) -> Result<(bool, bool, bool, bool), Box<dyn std::error::Error>> {
    configs::get_flags(pool, dedup_key).await
}

pub async fn mark_deleted(
    pool: &SqlitePool,
    dedup_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    configs::mark_deleted(pool, dedup_key).await
}

pub async fn set_selected_config(
    pool: &SqlitePool,
    id: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    configs::set_selected(pool, id).await
}

pub async fn set_active_config(
    pool: &SqlitePool,
    id: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    configs::set_active(pool, id).await
}

pub async fn set_config_enabled(
    pool: &SqlitePool,
    id: i64,
    is_enabled: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    configs::set_enabled(pool, id, is_enabled).await
}

pub async fn soft_delete_config(
    pool: &SqlitePool,
    id: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    configs::soft_delete(pool, id).await
}

pub async fn restore_config(pool: &SqlitePool, id: i64) -> Result<(), Box<dyn std::error::Error>> {
    configs::restore(pool, id).await
}
