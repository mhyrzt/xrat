use super::super::Database;
use super::super::types::*;

impl Database {
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
