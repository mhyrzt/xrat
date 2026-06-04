use crate::db::connection::DbPool;
use crate::db::record::{
    ConfigListFilter, ConfigRecord, ConfigWithLatestTest, ImportSource, ImportSummary,
};
use crate::db::repository::{configs, subscriptions};
use crate::model::Node;

pub async fn import_nodes(
    pool: &DbPool,
    source: &ImportSource,
    nodes: &[Node],
) -> crate::db::Result<ImportSummary> {
    let subscription_id = subscriptions::find_or_create(pool, source).await?;
    configs::import_nodes(pool, subscription_id, nodes).await
}

pub async fn get_config_count(pool: &DbPool) -> crate::db::Result<i64> {
    configs::get_count(pool).await
}

pub async fn list_configs(
    pool: &DbPool,
    filter: &ConfigListFilter,
) -> crate::db::Result<Vec<ConfigRecord>> {
    configs::list(pool, filter).await
}

pub async fn get_config_by_id(pool: &DbPool, id: i64) -> crate::db::Result<Option<ConfigRecord>> {
    configs::get_by_id(pool, id).await
}

pub async fn get_active_config(pool: &DbPool) -> crate::db::Result<Option<ConfigRecord>> {
    configs::get_active(pool).await
}

pub async fn delete_config(pool: &DbPool, id: i64) -> crate::db::Result<()> {
    configs::soft_delete(pool, id).await
}

pub async fn restore_config(pool: &DbPool, id: i64) -> crate::db::Result<()> {
    configs::restore(pool, id).await
}

pub async fn hard_delete_config(pool: &DbPool, id: i64) -> crate::db::Result<()> {
    configs::hard_delete(pool, id).await
}

pub async fn set_active_config(pool: &DbPool, id: i64) -> crate::db::Result<()> {
    configs::clear_all_active(pool).await?;
    configs::mark_active(pool, id).await
}

pub async fn clear_active_config(pool: &DbPool) -> crate::db::Result<()> {
    configs::clear_all_active(pool).await
}

pub async fn set_config_enabled(pool: &DbPool, id: i64, is_enabled: bool) -> crate::db::Result<()> {
    configs::set_enabled(pool, id, is_enabled).await
}

pub async fn list_configs_with_latest_tests(
    pool: &DbPool,
    filter: &ConfigListFilter,
) -> crate::db::Result<Vec<ConfigWithLatestTest>> {
    configs::list_with_latest_tests(pool, filter).await
}

pub async fn list_top_configs_by_real_delay(
    pool: &DbPool,
    limit: i64,
    filter: &ConfigListFilter,
) -> crate::db::Result<Vec<ConfigWithLatestTest>> {
    configs::list_top_by_real_delay(pool, limit, filter).await
}

pub async fn count_filtered_configs(
    pool: &DbPool,
    filter: &ConfigListFilter,
) -> crate::db::Result<i64> {
    configs::count_filtered(pool, filter).await
}

pub async fn list_configs_paginated_with_latest_tests(
    pool: &DbPool,
    filter: &ConfigListFilter,
    offset: i64,
    limit: i64,
) -> crate::db::Result<Vec<ConfigWithLatestTest>> {
    configs::list_paginated_with_latest_tests(pool, filter, offset, limit).await
}

pub async fn get_config_with_latest_test(
    pool: &DbPool,
    id: i64,
) -> crate::db::Result<Option<ConfigWithLatestTest>> {
    configs::get_with_latest_test(pool, id).await
}
