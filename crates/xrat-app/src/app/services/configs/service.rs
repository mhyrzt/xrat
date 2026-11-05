use std::sync::Arc;

use crate::app::ports::ConfigRepository;
use crate::app::read_models::ConfigDetail;
use crate::app::{AppError, Result};
use xrat_db::record::{ConfigListFilter, RefMatch};
use xrat_model::{ConfigId, ConfigRef, SubscriptionId};

use super::repository::DatabaseConfigRepository;

/// Filter and pagination request for listing configs.
///
/// Adapters translate their own input (CLI args, HTTP query params, TUI state)
/// into this struct so filter defaults and pagination rules live in one place.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigListRequest {
    pub only_enabled: bool,
    pub only_active: bool,
    pub only_deleted: bool,
    pub include_deleted: bool,
    pub subscription_id: Option<SubscriptionId>,
    pub protocol: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub top_by_real_delay: Option<u32>,
}

impl ConfigListRequest {
    pub fn filter(&self) -> ConfigListFilter {
        ConfigListFilter {
            only_enabled: self.only_enabled,
            only_active: self.only_active,
            only_deleted: self.only_deleted,
            include_deleted: self.include_deleted,
            subscription_id: self.subscription_id,
            protocol: self.protocol.clone(),
        }
    }
}

/// Inclusive upper bound accepted for `top` selections.
pub const MAX_TOP: u32 = 200;

/// Validate a requested `top` count against the shared limit.
pub fn validate_top(top: u32) -> Result<u32> {
    if top == 0 || top > MAX_TOP {
        return Err(AppError::InvalidArgument(format!(
            "top must be between 1 and {MAX_TOP}"
        )));
    }
    Ok(top)
}

/// Result of a config list query.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConfigListResult {
    pub total: i64,
    pub items: Vec<ConfigDetail>,
}

/// Owns config read rules shared by CLI, HTTP, and TUI.
#[derive(Clone)]
pub struct ConfigService {
    repository: Arc<DatabaseConfigRepository>,
    #[cfg(any(windows, test))]
    machine_client: Option<crate::app::services::machine_service::MachineClient>,
}

impl ConfigService {
    pub fn new(repository: Arc<DatabaseConfigRepository>) -> Self {
        Self {
            repository,
            #[cfg(any(windows, test))]
            machine_client: None,
        }
    }

    #[cfg(any(windows, test))]
    pub(crate) fn for_machine(
        client: crate::app::services::machine_service::MachineClient,
    ) -> Self {
        Self {
            repository: Arc::new(DatabaseConfigRepository::detached()),
            machine_client: Some(client),
        }
    }

    /// Shared repository access for sibling services.
    /// Machine services expose a detached repository whose operations return Unsupported.
    pub fn repository(&self) -> Arc<DatabaseConfigRepository> {
        Arc::clone(&self.repository)
    }

    /// List configs with the latest test joined, applying filter and pagination.
    pub async fn list(&self, request: &ConfigListRequest) -> Result<ConfigListResult> {
        #[cfg(any(windows, test))]
        if let Some(client) = &self.machine_client {
            return client.list(request).await;
        }

        let filter = request.filter();
        let items = if let Some(top) = request.top_by_real_delay {
            self.repository
                .list_top_configs_by_real_delay(i64::from(top), &filter)
                .await?
        } else if let (Some(offset), Some(limit)) = (request.offset, request.limit) {
            self.repository
                .list_configs_paginated_with_latest_tests(&filter, offset, limit)
                .await?
        } else {
            self.repository
                .list_configs_with_latest_tests(&filter)
                .await?
        };

        let total = if request.top_by_real_delay.is_some()
            || (request.offset.is_none() && request.limit.is_none())
        {
            items.len() as i64
        } else {
            self.repository.count_filtered_configs(&filter).await?
        };

        let items = items.iter().map(ConfigDetail::from_joined).collect();
        Ok(ConfigListResult { total, items })
    }

    /// Raw config links for a filtered selection, in listing order.
    pub async fn export_raw_configs(&self, request: &ConfigListRequest) -> Result<Vec<String>> {
        #[cfg(any(windows, test))]
        if let Some(client) = &self.machine_client {
            return client.export(request).await;
        }

        let filter = request.filter();
        if let Some(top) = request.top_by_real_delay {
            let rows = self
                .repository
                .list_top_configs_by_real_delay(i64::from(top), &filter)
                .await?;
            let configs = rows.into_iter().map(|row| row.config).collect::<Vec<_>>();
            reject_profile_subscription(&configs)?;
            return Ok(configs.into_iter().map(|row| row.raw_config).collect());
        }

        let configs = self.repository.list_configs(&filter).await?;
        reject_profile_subscription(&configs)?;
        Ok(configs
            .into_iter()
            .map(|config| config.raw_config)
            .collect())
    }

    /// Fetch a config as an interface-neutral detail model.
    pub async fn detail(&self, id: ConfigId) -> Result<Option<ConfigDetail>> {
        #[cfg(any(windows, test))]
        if let Some(client) = &self.machine_client {
            return client.config_detail(id).await;
        }

        Ok(self
            .repository
            .get_config_with_latest_test(id)
            .await?
            .as_ref()
            .map(ConfigDetail::from_joined))
    }

    /// Raw links in explicit selection order. Missing IDs return no partial selection.
    pub async fn raw_configs_by_ids(&self, ids: &[ConfigId]) -> Result<Option<Vec<String>>> {
        #[cfg(any(windows, test))]
        if let Some(client) = &self.machine_client {
            return client.raw_configs_by_ids(ids).await;
        }
        match ids {
            [] => Ok(Some(Vec::new())),
            [id] => match self.repository.get_config_by_id(*id).await? {
                Some(row) => Ok(Some(vec![self.repository.raw_content(&row).await?])),
                None => Ok(None),
            },
            _ => {
                let raw = self
                    .repository
                    .list_configs(&ConfigListFilter {
                        include_deleted: true,
                        ..Default::default()
                    })
                    .await?
                    .into_iter()
                    .map(|row| (row.id, row))
                    .collect::<std::collections::HashMap<_, _>>();
                let Some(rows) = ids
                    .iter()
                    .map(|id| raw.get(id).cloned())
                    .collect::<Option<Vec<_>>>()
                else {
                    return Ok(None);
                };
                reject_profile_subscription(&rows)?;
                Ok(Some(rows.into_iter().map(|row| row.raw_config).collect()))
            }
        }
    }

    pub async fn reference(&self, id: ConfigId) -> Result<Option<String>> {
        #[cfg(any(windows, test))]
        if let Some(client) = &self.machine_client {
            if id.0 <= 0 {
                return Ok(None);
            }
            return client.config_reference(id).await;
        }

        Ok(self
            .repository
            .get_config_by_id(id)
            .await?
            .map(|record| record.r#ref))
    }

    /// Resolve a numeric id or ref prefix into a config id.
    pub async fn resolve_id(&self, token: &ConfigRef) -> Result<Option<ConfigId>> {
        #[cfg(any(windows, test))]
        if let Some(client) = &self.machine_client {
            return client.find_config(token).await;
        }

        let raw = token.as_ref();
        if let Ok(id) = raw.parse::<i64>() {
            let id = ConfigId(id);
            let exists = self.repository.get_config_by_id(id).await?.is_some();
            if exists {
                return Ok(Some(id));
            }
        }

        if !xrat_support::refs::is_ref_prefix(raw) {
            return Ok(None);
        }

        match self.repository.resolve_config_ref_prefix(raw).await? {
            RefMatch::Unique(id) => Ok(Some(id)),
            RefMatch::Ambiguous => Err(AppError::InvalidArgument(format!(
                "config ref prefix '{raw}' is ambiguous; provide more characters"
            ))),
            RefMatch::None => Ok(None),
        }
    }

    /// Resolve a numeric subscription id or ref prefix.
    pub async fn resolve_subscription_id(&self, raw: &str) -> Result<Option<SubscriptionId>> {
        #[cfg(any(windows, test))]
        if let Some(client) = &self.machine_client {
            return client.find_subscription(raw).await;
        }

        if let Ok(id) = raw.parse::<i64>() {
            let id = SubscriptionId(id);
            if self.repository.get_subscription_by_id(id).await?.is_some() {
                return Ok(Some(id));
            }
        }
        if !xrat_support::refs::is_ref_prefix(raw) {
            return Ok(None);
        }
        match self.repository.resolve_subscription_ref_prefix(raw).await? {
            RefMatch::Unique(id) => Ok(Some(id)),
            RefMatch::Ambiguous => Err(AppError::InvalidArgument(format!(
                "subscription ref prefix '{raw}' is ambiguous; provide more characters"
            ))),
            RefMatch::None => Ok(None),
        }
    }

    /// List subscriptions for ref enrichment.
    pub async fn subscriptions(&self) -> Result<Vec<xrat_db::SubscriptionRecord>> {
        #[cfg(any(windows, test))]
        if let Some(client) = &self.machine_client {
            return client.subscriptions().await;
        }

        Ok(self.repository.list_subscriptions().await?)
    }
}

fn reject_profile_subscription(configs: &[xrat_db::ConfigRecord]) -> Result<()> {
    if configs.iter().any(xrat_db::ConfigRecord::is_xray_profile) {
        return Err(AppError::InvalidArgument("complete Xray profiles cannot be encoded as a share-link subscription; export each profile individually".into()));
    }
    Ok(())
}
