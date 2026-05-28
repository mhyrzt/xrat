use super::Database;
use super::types::*;

impl Database {
    pub async fn connect(config: &DatabaseConnectionConfig) -> crate::db::Result<Self> {
        let pool = connection::connect(config).await?;
        Ok(Self { pool })
    }

    #[cfg(test)]
    pub(super) async fn connect_sqlite(database_path: &std::path::Path) -> crate::db::Result<Self> {
        Self::connect(&DatabaseConnectionConfig::Sqlite {
            path: database_path.to_path_buf(),
        })
        .await
    }

    #[cfg(test)]
    pub(super) async fn connect_postgres_url(url: String) -> crate::db::Result<Self> {
        Self::connect(&DatabaseConnectionConfig::Postgres {
            url,
            max_connections: 5,
            min_connections: 0,
            connect_timeout: Duration::from_secs(10),
        })
        .await
    }

    #[cfg(test)]
    pub(super) async fn clear_for_test(&self) -> crate::db::Result<()> {
        match &self.pool {
            DbPool::Sqlite(pool) => {
                sqlx::query("DELETE FROM runtime_sessions")
                    .execute(pool)
                    .await?;
                sqlx::query("DELETE FROM connection_tests")
                    .execute(pool)
                    .await?;
                sqlx::query("DELETE FROM connection_test_runs")
                    .execute(pool)
                    .await?;
                sqlx::query("DELETE FROM cf_scan_results")
                    .execute(pool)
                    .await?;
                sqlx::query("DELETE FROM configs").execute(pool).await?;
                sqlx::query("DELETE FROM subscriptions")
                    .execute(pool)
                    .await?;
            }
            DbPool::Postgres(pool) => {
                sqlx::query(
                    "TRUNCATE runtime_sessions, connection_tests, connection_test_runs, cf_scan_results, configs, subscriptions RESTART IDENTITY CASCADE",
                )
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }

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

    pub async fn get_selected_config(&self) -> crate::db::Result<Option<ConfigRecord>> {
        repository::get_selected_config(&self.pool).await
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

    pub async fn get_connection_test_count(&self) -> crate::db::Result<i64> {
        repository::get_connection_test_count(&self.pool).await
    }

    pub async fn get_config_flags(&self, dedup_key: &str) -> crate::db::Result<(bool, bool, bool)> {
        repository::get_config_flags(&self.pool, dedup_key).await
    }

    pub async fn delete_config(&self, id: i64) -> crate::db::Result<()> {
        repository::delete_config(&self.pool, id).await
    }

    pub async fn restore_config(&self, id: i64) -> crate::db::Result<()> {
        repository::restore_config(&self.pool, id).await
    }

    pub async fn hard_delete_config(&self, id: i64) -> crate::db::Result<()> {
        repository::hard_delete_config(&self.pool, id).await
    }

    pub async fn set_selected_config(&self, id: i64) -> crate::db::Result<()> {
        repository::set_selected_config(&self.pool, id).await
    }

    pub async fn set_active_config(&self, id: i64) -> crate::db::Result<()> {
        repository::set_active_config(&self.pool, id).await
    }

    pub async fn clear_active_config(&self) -> crate::db::Result<()> {
        repository::clear_active_config(&self.pool).await
    }

    pub async fn set_config_enabled(&self, id: i64, is_enabled: bool) -> crate::db::Result<()> {
        repository::set_config_enabled(&self.pool, id, is_enabled).await
    }

    pub async fn list_configs_with_latest_tests(
        &self,
        filter: &ConfigListFilter,
    ) -> crate::db::Result<Vec<ConfigWithLatestTest>> {
        repository::list_configs_with_latest_tests(&self.pool, filter).await
    }

    pub async fn list_top_configs_by_real_delay(
        &self,
        limit: i64,
        filter: &ConfigListFilter,
    ) -> crate::db::Result<Vec<ConfigWithLatestTest>> {
        repository::list_top_configs_by_real_delay(&self.pool, limit, filter).await
    }

    pub async fn count_filtered_configs(
        &self,
        filter: &ConfigListFilter,
    ) -> crate::db::Result<i64> {
        repository::count_filtered_configs(&self.pool, filter).await
    }

    pub async fn list_configs_paginated_with_latest_tests(
        &self,
        filter: &ConfigListFilter,
        offset: i64,
        limit: i64,
    ) -> crate::db::Result<Vec<ConfigWithLatestTest>> {
        repository::list_configs_paginated_with_latest_tests(&self.pool, filter, offset, limit)
            .await
    }

    pub async fn get_config_with_latest_test(
        &self,
        id: i64,
    ) -> crate::db::Result<Option<ConfigWithLatestTest>> {
        repository::get_config_with_latest_test(&self.pool, id).await
    }
}
