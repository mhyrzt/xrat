use crate::db::connection::DbPool;
use crate::db::record::SubscriptionRecord;
use crate::db::repository::subscriptions;

pub async fn get_subscription_count(pool: &DbPool) -> crate::db::Result<i64> {
    subscriptions::get_count(pool).await
}

pub async fn list_subscriptions(pool: &DbPool) -> crate::db::Result<Vec<SubscriptionRecord>> {
    subscriptions::list(pool).await
}
