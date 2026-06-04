use crate::db::{ConnectionTestRecord, ConnectionTestRunRecord};

use super::TuiConfigRow;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TuiTestStatus {
    pub latest_run_id: Option<i64>,
    pub latest_run_kind: Option<String>,
    pub latest_run_created_at: Option<String>,
    pub total_results: usize,
    pub success_results: usize,
    pub failed_results: usize,
    pub untested_configs: usize,
    pub stale_configs: usize,
}

impl TuiTestStatus {
    pub(super) async fn load(
        context: &crate::app::context::AppContext,
        configs: &[TuiConfigRow],
    ) -> crate::app::Result<Self> {
        let latest_run = context.db.get_latest_connection_test_run().await?;
        let results = match latest_run.as_ref() {
            Some(run) => context.db.list_connection_tests_by_run(run.id).await?,
            None => Vec::new(),
        };

        Ok(Self::from_run_and_results(latest_run, results, configs))
    }

    pub fn from_run_and_results(
        run: Option<ConnectionTestRunRecord>,
        results: Vec<ConnectionTestRecord>,
        configs: &[TuiConfigRow],
    ) -> Self {
        let total_results = results.len();
        let failed_results = results
            .iter()
            .filter(|result| result.failure_reason.is_some() || result.failure_kind.is_some())
            .count();
        let success_results = total_results.saturating_sub(failed_results);
        let untested_configs = configs
            .iter()
            .filter(|config| config.real_delay_ms.is_none() && config.tcp_ms.is_none())
            .count();
        let stale_configs = configs
            .iter()
            .filter(|config| config.failure_reason.is_some())
            .count();

        Self {
            latest_run_id: run.as_ref().map(|run| run.id),
            latest_run_kind: run.as_ref().map(|run| run.kind.clone()),
            latest_run_created_at: run.map(|run| run.created_at),
            total_results,
            success_results,
            failed_results,
            untested_configs,
            stale_configs,
        }
    }

    pub fn progress_label(&self) -> String {
        match self.latest_run_id {
            Some(_) => format!(
                "{} done · {} ok · {} failed",
                self.total_results, self.success_results, self.failed_results
            ),
            None => "no test run yet".to_string(),
        }
    }
}
