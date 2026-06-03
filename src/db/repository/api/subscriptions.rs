use crate::db::connection::DbPool;
use crate::db::record::SubscriptionRecord;
use crate::db::repository::subscriptions;

pub async fn get_subscription_count(pool: &DbPool) -> crate::db::Result<i64> {
    subscriptions::get_count(pool).await
}

pub async fn list_subscriptions(pool: &DbPool) -> crate::db::Result<Vec<SubscriptionRecord>> {
    subscriptions::list(pool).await
}

pub async fn set_subscription_name(pool: &DbPool, id: i64, name: &str) -> crate::db::Result<()> {
    subscriptions::set_name(pool, id, name).await
}

pub async fn delete_subscription_with_configs(pool: &DbPool, id: i64) -> crate::db::Result<()> {
    subscriptions::delete_with_configs(pool, id).await
}
