use super::super::Database;
use super::super::types::*;

impl Database {
    pub async fn import_nodes(
        &self,
        source: &ImportSource,
        nodes: &[crate::model::Node],
    ) -> crate::db::Result<ImportSummary> {
        repository::import_nodes(&self.pool, source, nodes).await
    }

    pub async fn get_config_count(&self) -> crate::db::Result<i64> {
        repository::get_config_count(&self.pool).await
    }

    pub async fn list_configs(
        &self,
        filter: &ConfigListFilter,
    ) -> crate::db::Result<Vec<ConfigRecord>> {
        repository::list_configs(&self.pool, filter).await
    }

    pub async fn get_config_by_id(&self, id: i64) -> crate::db::Result<Option<ConfigRecord>> {
        repository::get_config_by_id(&self.pool, id).await
    }

    pub async fn resolve_config_ref_prefix(&self, prefix: &str) -> crate::db::Result<RefMatch> {
        repository::resolve_config_ref_prefix(&self.pool, prefix).await
    }

    pub async fn resolve_subscription_ref_prefix(
        &self,
        prefix: &str,
    ) -> crate::db::Result<RefMatch> {
        repository::resolve_subscription_ref_prefix(&self.pool, prefix).await
    }

    pub async fn get_active_config(&self) -> crate::db::Result<Option<ConfigRecord>> {
        repository::get_active_config(&self.pool).await
    }

    pub async fn get_subscription_count(&self) -> crate::db::Result<i64> {
        repository::get_subscription_count(&self.pool).await
    }

    pub async fn list_subscriptions(&self) -> crate::db::Result<Vec<SubscriptionRecord>> {
        repository::list_subscriptions(&self.pool).await
    }

    pub async fn list_refreshable_due_subscriptions(
        &self,
        cutoff_epoch_secs: i64,
    ) -> crate::db::Result<Vec<RefreshableSubscription>> {
        repository::list_refreshable_due_subscriptions(&self.pool, cutoff_epoch_secs).await
    }

    pub async fn get_subscription_by_id(
        &self,
        id: i64,
    ) -> crate::db::Result<Option<SubscriptionRecord>> {
        repository::get_subscription_by_id(&self.pool, id).await
    }

    pub async fn set_subscription_name(&self, id: i64, name: &str) -> crate::db::Result<()> {
        repository::set_subscription_name(&self.pool, id, name).await
    }

    pub async fn delete_subscription_with_configs(&self, id: i64) -> crate::db::Result<()> {
        repository::delete_subscription_with_configs(&self.pool, id).await
    }

    pub async fn get_connection_test_count(&self) -> crate::db::Result<i64> {
        repository::get_connection_test_count(&self.pool).await
    }
}
