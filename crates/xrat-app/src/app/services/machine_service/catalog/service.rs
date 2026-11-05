use super::{CatalogPayload, CatalogRequest, policy};
use crate::app::context::AppContext;
use crate::app::{Result, import};

pub(crate) struct MachineCatalog {
    context: AppContext,
}

impl MachineCatalog {
    #[cfg(any(windows, test))]
    pub(crate) fn new(context: AppContext) -> Self {
        Self { context }
    }

    pub(crate) async fn dispatch(&self, request: CatalogRequest) -> Result<CatalogPayload> {
        policy::validate(&request)?;
        let services = self.context.services();
        Ok(match request {
            CatalogRequest::RawConfigs { ids } => {
                CatalogPayload::RawConfigs(services.configs.raw_configs_by_ids(&ids).await?)
            }
            CatalogRequest::Detail { config_id } => {
                CatalogPayload::Detail(services.configs.detail(config_id).await?.map(Box::new))
            }
            CatalogRequest::FindConfig { reference } => {
                CatalogPayload::FoundConfig(services.configs.resolve_id(&reference).await?)
            }
            CatalogRequest::FindSubscription { reference } => CatalogPayload::FoundSubscription(
                services.configs.resolve_subscription_id(&reference).await?,
            ),
            CatalogRequest::Purge {} => CatalogPayload::Changed(services.lifecycle.purge().await?),
            CatalogRequest::Profile {} => CatalogPayload::Profile {
                database_label: self.context.runtime_paths.database_label.clone(),
                config_path: self.context.runtime_paths.config_path.display().to_string(),
            },
            CatalogRequest::List { filter } => {
                CatalogPayload::Configs(self.list_page(&filter).await?)
            }
            CatalogRequest::Config { reference } => {
                CatalogPayload::Config(Box::new(services.lifecycle.config(&reference).await?))
            }
            CatalogRequest::ResolveConfig { reference } => {
                CatalogPayload::ConfigId(services.lifecycle.resolve_config_id(&reference).await?)
            }
            CatalogRequest::Reference { config_id } => {
                CatalogPayload::Reference(services.configs.reference(config_id).await?)
            }
            CatalogRequest::ResolveSubscription { reference } => CatalogPayload::SubscriptionId(
                services
                    .lifecycle
                    .resolve_subscription_id(&reference)
                    .await?,
            ),
            CatalogRequest::Subscriptions { offset, limit } => {
                let subscriptions = services.configs.subscriptions().await?;
                let total = subscriptions.len() as u64;
                CatalogPayload::Subscriptions {
                    total,
                    items: subscriptions
                        .into_iter()
                        .skip(offset as usize)
                        .take(limit as usize)
                        .collect(),
                }
            }
            CatalogRequest::Subscription { reference } => CatalogPayload::Subscription(Box::new(
                services.lifecycle.subscription(&reference).await?,
            )),
            CatalogRequest::Import {
                contents,
                source,
                name,
                headers,
            } => {
                let mut nodes = Vec::new();
                for content in contents {
                    if crate::app::services::xray_profiles::parse(content.as_bytes())?.is_some() {
                        return Err(crate::app::AppError::InvalidArgument("Complete Xray profiles cannot be provisioned through the machine gateway".into()));
                    }
                    nodes.extend(import::parse_nodes(content.as_bytes())?);
                }
                policy::validate_nodes(&nodes)?;
                let stored_headers =
                    matches!(source.kind, xrat_db::SourceKind::Url).then_some(&headers);
                CatalogPayload::Import(
                    import::persist_nodes(
                        &self.context.db,
                        source,
                        &nodes,
                        name.as_deref(),
                        stored_headers,
                    )
                    .await?,
                )
            }
            CatalogRequest::Enable { reference } => {
                let outcome = services.lifecycle.enable(&reference).await?;
                CatalogPayload::Toggle {
                    config: Box::new(services.lifecycle.config(&reference).await?),
                    outcome,
                }
            }
            CatalogRequest::Disable { reference } => {
                let outcome = services.lifecycle.disable(&reference).await?;
                CatalogPayload::Toggle {
                    config: Box::new(services.lifecycle.config(&reference).await?),
                    outcome,
                }
            }
            CatalogRequest::Delete { reference, hard } => {
                let (config, outcome) = services.lifecycle.delete(&reference, hard).await?;
                CatalogPayload::Delete {
                    config: Box::new(config),
                    outcome,
                }
            }
            CatalogRequest::Restore { reference } => {
                let (config, outcome) = services.lifecycle.restore(&reference).await?;
                CatalogPayload::Restore {
                    config: Box::new(config),
                    outcome,
                }
            }
            CatalogRequest::DeleteMany { ids, hard } => {
                CatalogPayload::Changed(services.lifecycle.delete_many(&ids, hard).await?)
            }
            CatalogRequest::RestoreMany { ids } => {
                CatalogPayload::Changed(services.lifecycle.restore_many(&ids).await?)
            }
            CatalogRequest::DeleteSubscription { reference } => CatalogPayload::Subscription(
                Box::new(services.lifecycle.delete_subscription(&reference).await?),
            ),
            CatalogRequest::RenameSubscription { reference, name } => {
                CatalogPayload::Subscription(Box::new(
                    services
                        .lifecycle
                        .rename_subscription(&reference, &name)
                        .await?,
                ))
            }
            CatalogRequest::ExportPage { filter } => {
                let page = self.list_page(&filter).await?;
                let mut items = Vec::with_capacity(page.items.len());
                for item in page.items {
                    if item.summary.protocol == xrat_db::XRAY_PROFILE_PROTOCOL {
                        return Err(crate::app::AppError::InvalidArgument(
                            "complete Xray profiles cannot be exported as a subscription".into(),
                        ));
                    }
                    items.extend(
                        services
                            .configs
                            .raw_configs_by_ids(&[item.summary.id])
                            .await?
                            .unwrap_or_default(),
                    );
                }
                CatalogPayload::ExportPage {
                    total: page.total,
                    items,
                }
            }
        })
    }

    async fn list_page(
        &self,
        filter: &crate::app::services::ConfigListRequest,
    ) -> Result<crate::app::services::ConfigListResult> {
        let mut page = self.context.services().configs.list(filter).await?;
        if let (Some(_), Some(offset), Some(limit)) =
            (filter.top_by_real_delay, filter.offset, filter.limit)
        {
            page.items = page
                .items
                .into_iter()
                .skip(offset as usize)
                .take(limit as usize)
                .collect();
        }
        Ok(page)
    }
}
