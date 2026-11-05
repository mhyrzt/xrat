use crate::app::ports::ConfigRepository;
use xrat_db::record::{ConfigListFilter, ConfigWithLatestTest, RefMatch};
use xrat_db::{ConfigRecord, Database, DbError, SubscriptionRecord};
use xrat_model::{ConfigId, SubscriptionId};

/// Adapter that implements [`ConfigRepository`] over the database facade.
///
/// Kept in the application layer so the database module stays unaware of
/// application ports.
#[derive(Clone)]
pub struct DatabaseConfigRepository {
    db: Option<Database>,
    profiles: Option<crate::app::services::xray_profiles::ProfileStore>,
}

impl DatabaseConfigRepository {
    pub fn new(db: Database) -> Self {
        Self {
            db: Some(db),
            profiles: None,
        }
    }
    #[cfg(any(windows, test))]
    pub(crate) fn detached() -> Self {
        Self {
            db: None,
            profiles: None,
        }
    }

    pub(crate) fn with_profiles(
        mut self,
        store: crate::app::services::xray_profiles::ProfileStore,
    ) -> Self {
        self.profiles = Some(store);
        self
    }

    pub(crate) async fn raw_content(&self, config: &ConfigRecord) -> crate::app::Result<String> {
        if !config.is_xray_profile() {
            return Ok(config.raw_config.clone());
        }
        let store = self.profiles.as_ref().ok_or_else(|| {
            crate::app::AppError::InvalidArgument(
                "Xray profile export requires the owning application context".into(),
            )
        })?;
        let metadata = self
            .database()?
            .get_xray_profile(config.id)
            .await?
            .ok_or_else(|| {
                crate::app::AppError::InvalidArgument("Xray profile metadata is missing".into())
            })?;
        String::from_utf8(store.read(&metadata)?.as_bytes().to_vec()).map_err(|_| {
            crate::app::AppError::InvalidArgument("Xray profile must contain UTF-8 JSON".into())
        })
    }

    fn profile_lock(&self) -> Result<Option<std::fs::File>, DbError> {
        self.profiles
            .as_ref()
            .map(|store| {
                store
                    .lock()
                    .map_err(|error| std::io::Error::other(error.to_string()).into())
            })
            .transpose()
    }

    async fn cleanup_profiles(&self) -> Result<(), DbError> {
        if let Some(store) = &self.profiles {
            let retained = self.database()?.xray_profile_digests().await?;
            store
                .cleanup(&retained)
                .map_err(|error| std::io::Error::other(error.to_string()))?;
        }
        Ok(())
    }

    async fn protect_running_profile(&self, ids: &[ConfigId]) -> Result<(), DbError> {
        if self.profiles.is_none()
            && self
                .database()?
                .list_configs(&ConfigListFilter {
                    include_deleted: true,
                    ..Default::default()
                })
                .await?
                .iter()
                .any(|config| config.is_xray_profile() && ids.contains(&config.id))
        {
            return Err(std::io::Error::other(
                "permanent Xray profile deletion requires the owning application context",
            )
            .into());
        }
        if let Some(session) = self.database()?.get_running_runtime_session().await?
            && let Some(id) = session.config_id.filter(|id| ids.contains(id))
            && self
                .database()?
                .get_config_by_id(id)
                .await?
                .is_some_and(|config| config.is_xray_profile())
        {
            return Err(std::io::Error::other(
                "disconnect the complete Xray profile before permanently deleting it",
            )
            .into());
        }
        Ok(())
    }

    fn database(&self) -> Result<&Database, DbError> {
        self.db.as_ref().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "machine catalog persistence is available only through the service gateway",
            )
            .into()
        })
    }
}

#[async_trait::async_trait]
impl ConfigRepository for DatabaseConfigRepository {
    async fn list_configs_with_latest_tests(
        &self,
        filter: &ConfigListFilter,
    ) -> Result<Vec<ConfigWithLatestTest>, DbError> {
        self.database()?
            .list_configs_with_latest_tests(filter)
            .await
    }

    async fn list_configs_paginated_with_latest_tests(
        &self,
        filter: &ConfigListFilter,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<ConfigWithLatestTest>, DbError> {
        self.database()?
            .list_configs_paginated_with_latest_tests(filter, offset, limit)
            .await
    }

    async fn get_config_with_latest_test(
        &self,
        id: ConfigId,
    ) -> Result<Option<ConfigWithLatestTest>, DbError> {
        self.database()?.get_config_with_latest_test(id).await
    }

    async fn count_filtered_configs(&self, filter: &ConfigListFilter) -> Result<i64, DbError> {
        self.database()?.count_filtered_configs(filter).await
    }

    async fn list_top_configs_by_real_delay(
        &self,
        limit: i64,
        filter: &ConfigListFilter,
    ) -> Result<Vec<ConfigWithLatestTest>, DbError> {
        self.database()?
            .list_top_configs_by_real_delay(limit, filter)
            .await
    }

    async fn list_configs(&self, filter: &ConfigListFilter) -> Result<Vec<ConfigRecord>, DbError> {
        self.database()?.list_configs(filter).await
    }

    async fn resolve_config_ref_prefix(&self, prefix: &str) -> Result<RefMatch<ConfigId>, DbError> {
        self.database()?.resolve_config_ref_prefix(prefix).await
    }

    async fn get_config_by_id(&self, id: ConfigId) -> Result<Option<ConfigRecord>, DbError> {
        self.database()?.get_config_by_id(id).await
    }

    async fn list_subscriptions(&self) -> Result<Vec<SubscriptionRecord>, DbError> {
        self.database()?.list_subscriptions().await
    }

    async fn resolve_subscription_ref_prefix(
        &self,
        prefix: &str,
    ) -> Result<RefMatch<SubscriptionId>, DbError> {
        self.database()?
            .resolve_subscription_ref_prefix(prefix)
            .await
    }

    async fn get_subscription_by_id(
        &self,
        id: SubscriptionId,
    ) -> Result<Option<SubscriptionRecord>, DbError> {
        self.database()?.get_subscription_by_id(id).await
    }

    async fn set_config_enabled(&self, id: ConfigId, is_enabled: bool) -> Result<(), DbError> {
        self.database()?.set_config_enabled(id, is_enabled).await
    }

    async fn delete_config(&self, id: ConfigId) -> Result<(), DbError> {
        self.database()?.delete_config(id).await
    }

    async fn hard_delete_config(&self, id: ConfigId) -> Result<(), DbError> {
        let _lock = self.profile_lock()?;
        self.protect_running_profile(&[id]).await?;
        self.database()?.hard_delete_config(id).await?;
        self.cleanup_profiles().await
    }

    async fn restore_config(&self, id: ConfigId) -> Result<(), DbError> {
        self.database()?.restore_config(id).await
    }

    async fn delete_subscription_with_configs(&self, id: SubscriptionId) -> Result<(), DbError> {
        let _lock = self.profile_lock()?;
        let ids = self
            .database()?
            .list_configs(&ConfigListFilter {
                subscription_id: Some(id),
                include_deleted: true,
                ..Default::default()
            })
            .await?
            .into_iter()
            .map(|config| config.id)
            .collect::<Vec<_>>();
        self.protect_running_profile(&ids).await?;
        self.database()?
            .delete_subscription_with_configs(id)
            .await?;
        self.cleanup_profiles().await
    }
    async fn set_subscription_name(&self, id: SubscriptionId, name: &str) -> Result<(), DbError> {
        self.database()?.set_subscription_name(id, name).await
    }
    async fn delete_configs(&self, ids: &[ConfigId]) -> Result<u64, DbError> {
        self.database()?.delete_configs(ids).await
    }
    async fn hard_delete_configs(&self, ids: &[ConfigId]) -> Result<u64, DbError> {
        let _lock = self.profile_lock()?;
        self.protect_running_profile(ids).await?;
        let count = self.database()?.hard_delete_configs(ids).await?;
        self.cleanup_profiles().await?;
        Ok(count)
    }
    async fn restore_configs(&self, ids: &[ConfigId]) -> Result<u64, DbError> {
        self.database()?.restore_configs(ids).await
    }
    async fn purge_deleted_configs(&self) -> Result<u64, DbError> {
        let _lock = self.profile_lock()?;
        let ids = self
            .database()?
            .list_configs(&ConfigListFilter {
                only_deleted: true,
                ..Default::default()
            })
            .await?
            .into_iter()
            .map(|config| config.id)
            .collect::<Vec<_>>();
        self.protect_running_profile(&ids).await?;
        let count = self.database()?.purge_deleted_configs().await?;
        self.cleanup_profiles().await?;
        Ok(count)
    }
}
