use crate::db::connection::DbPool;
use crate::db::record::{RefMatch, RefreshableSubscription, SubscriptionRecord};
use crate::db::repository::subscriptions;

pub async fn resolve_subscription_ref_prefix(
    pool: &DbPool,
    prefix: &str,
) -> crate::db::Result<RefMatch> {
    subscriptions::resolve_ref_prefix(pool, prefix).await
}

pub async fn get_subscription_count(pool: &DbPool) -> crate::db::Result<i64> {
    subscriptions::get_count(pool).await
}

pub async fn list_refreshable_due_subscriptions(
    pool: &DbPool,
    cutoff_epoch_secs: i64,
) -> crate::db::Result<Vec<RefreshableSubscription>> {
    subscriptions::list_refreshable_due(pool, cutoff_epoch_secs).await
}

pub async fn list_subscriptions(pool: &DbPool) -> crate::db::Result<Vec<SubscriptionRecord>> {
    subscriptions::list(pool).await
}

pub async fn get_subscription_by_id(
    pool: &DbPool,
    id: i64,
) -> crate::db::Result<Option<SubscriptionRecord>> {
    subscriptions::get_by_id(pool, id).await
}

pub async fn set_subscription_name(pool: &DbPool, id: i64, name: &str) -> crate::db::Result<()> {
    subscriptions::set_name(pool, id, name).await
}

pub async fn delete_subscription_with_configs(pool: &DbPool, id: i64) -> crate::db::Result<()> {
    subscriptions::delete_with_configs(pool, id).await
}
