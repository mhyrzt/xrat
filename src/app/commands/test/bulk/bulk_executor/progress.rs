use super::*;

pub(super) fn bulk_progress_bar(total: usize, enabled: bool) -> Option<ProgressBar> {
    if !enabled {
        return None;
    }

    let progress = ProgressBar::new(total as u64);
    let style = ProgressStyle::with_template(
        "{spinner:.green} testing [{bar:32.cyan/blue}] {pos}/{len} failed:{msg}",
    )
    .unwrap_or_else(|_| ProgressStyle::default_bar())
    .progress_chars("=>-");
    progress.set_style(style);
    progress.set_message("0");

    Some(progress)
}

pub(super) fn update_bulk_progress(
    progress: &Option<ProgressBar>,
    completed: usize,
    failed: usize,
) {
    if let Some(progress) = progress {
        progress.set_position(completed as u64);
        progress.set_message(failed.to_string());
    }
}

pub(super) fn finish_bulk_progress(progress: Option<ProgressBar>, completed: usize, failed: usize) {
    if let Some(progress) = progress {
        progress.finish_with_message(format!("done: {completed} tested, {failed} failed"));
    }
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
