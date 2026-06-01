mod configs;
mod runtime;
mod sources;
mod tests_view;

pub use configs::TuiConfigRow;
pub use runtime::TuiRuntimeStatus;
pub use sources::TuiSourceRow;
pub use tests_view::{TuiTestResultRow, TuiTestStatus};

use crate::app::runtime_service::RuntimeService;
use crate::db::ConfigListFilter;

#[derive(Debug, Default)]
pub struct TuiData {
    pub configs: Vec<TuiConfigRow>,
    pub sources: Vec<TuiSourceRow>,
    pub runtime: TuiRuntimeStatus,
    pub tests: TuiTestStatus,
    pub total_configs: usize,
    pub enabled_configs: usize,
    pub selected_configs: usize,
    pub deleted_configs: usize,
    pub failed_configs: usize,
}

impl TuiData {
    pub async fn load(
        context: &crate::app::context::AppContext,
        include_deleted: bool,
    ) -> crate::app::Result<Self> {
        let filter = ConfigListFilter {
            include_deleted,
            ..ConfigListFilter::default()
        };
        let mut configs: Vec<_> = context
            .db
            .list_configs_with_latest_tests(&filter)
            .await?
            .into_iter()
            .map(TuiConfigRow::from)
            .collect();

        configs.sort_by_key(|row| (row.real_delay_ms.unwrap_or(i64::MAX), row.id));
        let sources = context
            .db
            .list_subscriptions()
            .await?
            .into_iter()
            .map(TuiSourceRow::from)
            .collect();
        let runtime = RuntimeService::new(context).status().await?.into();
        let tests = TuiTestStatus::load(context, &configs).await?;

        Ok(Self::from_parts(configs, sources, runtime, tests))
    }

    pub fn from_configs(configs: Vec<TuiConfigRow>) -> Self {
        Self::from_configs_and_sources(configs, Vec::new())
    }

    pub fn from_configs_and_sources(
        configs: Vec<TuiConfigRow>,
        sources: Vec<TuiSourceRow>,
    ) -> Self {
        Self::from_parts(
            configs,
            sources,
            TuiRuntimeStatus::default(),
            TuiTestStatus::default(),
        )
    }

    pub fn from_parts(
        configs: Vec<TuiConfigRow>,
        sources: Vec<TuiSourceRow>,
        runtime: TuiRuntimeStatus,
        tests: TuiTestStatus,
    ) -> Self {
        let total_configs = configs.len();
        let enabled_configs = configs.iter().filter(|row| row.is_enabled).count();
        let selected_configs = configs.iter().filter(|row| row.is_selected).count();
        let deleted_configs = configs.iter().filter(|row| row.is_deleted).count();
        let failed_configs = configs
            .iter()
            .filter(|row| row.failure_reason.is_some())
            .count();

        Self {
            configs,
            sources,
            runtime,
            tests,
            total_configs,
            enabled_configs,
            selected_configs,
            deleted_configs,
            failed_configs,
        }
    }
}

#[cfg(test)]
mod tests;
