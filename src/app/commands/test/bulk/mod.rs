use super::*;

mod distribution;
mod runners;

pub(super) fn print_geo_distribution<'a>(label: &str, values: impl Iterator<Item = &'a str>) {
    distribution::print_geo_distribution(label, values);
}

pub(super) async fn run_single(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
    config_id: i64,
) -> crate::app::Result<()> {
    runners::run_single(args, context, settings, config_id).await
}

pub(super) async fn run_bulk(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
) -> crate::app::Result<()> {
    runners::run_bulk(args, context, settings).await
}

pub(super) fn bulk_progress_bar(total: usize, enabled: bool) -> Option<ProgressBar> {
    runners::bulk_progress_bar(total, enabled)
}

pub(super) fn update_bulk_progress(
    progress: &Option<ProgressBar>,
    completed: usize,
    failed: usize,
) {
    runners::update_bulk_progress(progress, completed, failed);
}

pub(super) fn finish_bulk_progress(progress: Option<ProgressBar>, completed: usize, failed: usize) {
    runners::finish_bulk_progress(progress, completed, failed);
}

pub(super) fn spawn_config_test(
    join_set: &mut JoinSet<crate::app::Result<TestOutputRow>>,
    db: Database,
    config: ConfigRecord,
    settings: ResolvedTestSettings,
    run_id: Option<i64>,
) {
    runners::spawn_config_test(join_set, db, config, settings, run_id);
}
