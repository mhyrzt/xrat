mod configs;
mod connection_tests;
mod runtime_sessions;
mod subscriptions;

use sqlx::SqlitePool;

use crate::db::models::{ImportSource, ImportSummary};
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

pub async fn get_subscription_count(pool: &SqlitePool) -> Result<i64, Box<dyn std::error::Error>> {
    subscriptions::get_count(pool).await
}

pub async fn get_connection_test_count(
    pool: &SqlitePool,
) -> Result<i64, Box<dyn std::error::Error>> {
    connection_tests::get_count(pool).await
}

pub async fn get_runtime_session_count(
    pool: &SqlitePool,
) -> Result<i64, Box<dyn std::error::Error>> {
    runtime_sessions::get_count(pool).await
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
