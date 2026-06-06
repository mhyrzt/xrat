use super::*;
use crate::app::commands::progress::CliProgress;

pub(super) fn bulk_progress_bar(total: usize, enabled: bool) -> CliProgress {
    CliProgress::bar_with_template(
        enabled,
        total as u64,
        "0",
        "{spinner:.green} testing [{bar:32.cyan/blue}] {pos}/{len} failed:{msg}",
    )
}

pub(super) fn update_bulk_progress(progress: &CliProgress, completed: usize, failed: usize) {
    progress.set_position(completed as u64);
    progress.set_message(failed.to_string());
}

pub(super) fn finish_bulk_progress(progress: CliProgress, completed: usize, failed: usize) {
    progress.finish_with_message(format!("done: {completed} tested, {failed} failed"));
}

pub(super) fn spawn_config_test(
    join_set: &mut JoinSet<crate::app::Result<TestOutputRow>>,
    db: Database,
    config: ConfigRecord,
    settings: ResolvedTestSettings,
    run_id: Option<i64>,
) {
    join_set
        .spawn(async move { test_and_record_config(db, config, settings, false, run_id).await });
}
