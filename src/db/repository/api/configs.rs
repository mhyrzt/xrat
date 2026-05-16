use crate::db::connection::DbPool;
use crate::db::record::{ConfigListFilter, ConfigRecord, ImportSource, ImportSummary};
use crate::db::repository::{configs, subscriptions};
use crate::model::Node;

pub async fn import_nodes(
    pool: &DbPool,
    source: &ImportSource,
    nodes: &[Node],
) -> crate::db::Result<ImportSummary> {
    let subscription_id = subscriptions::insert(pool, source).await?;
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

pub async fn get_selected_config(pool: &DbPool) -> crate::db::Result<Option<ConfigRecord>> {
    configs::get_selected(pool).await
}

pub async fn get_active_config(pool: &DbPool) -> crate::db::Result<Option<ConfigRecord>> {
    configs::get_active(pool).await
}

pub async fn get_config_flags(
    pool: &DbPool,
    dedup_key: &str,
) -> crate::db::Result<(bool, bool, bool)> {
    configs::get_flags(pool, dedup_key).await
}

pub async fn delete_config(pool: &DbPool, id: i64) -> crate::db::Result<()> {
    configs::delete(pool, id).await
}

pub async fn set_selected_config(pool: &DbPool, id: i64) -> crate::db::Result<()> {
    configs::clear_all_selected(pool).await?;
    configs::mark_selected(pool, id).await
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
