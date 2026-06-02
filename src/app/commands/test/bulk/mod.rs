use super::*;

mod bulk_executor;
mod distribution;
mod rotation;

pub(crate) use rotation::run_rotation_bulk_tests;

pub(super) fn print_geo_distribution<'a>(label: &str, values: impl Iterator<Item = &'a str>) {
    distribution::print_geo_distribution(label, values);
}

pub(super) async fn run_single(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
    config_id: i64,
) -> crate::app::Result<()> {
    bulk_executor::run_single(args, context, settings, config_id).await
}

pub(super) async fn run_bulk(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
) -> crate::app::Result<()> {
    bulk_executor::run_bulk(args, context, settings).await
}

pub(crate) async fn run_bulk_for_configs(
    context: &AppContext,
    settings: ResolvedTestSettings,
    configs: Vec<ConfigRecord>,
    run_kind: &str,
    show_progress: bool,
) -> crate::app::Result<Vec<TestOutputRow>> {
    bulk_executor::run_bulk_for_configs(context, settings, configs, run_kind, show_progress).await
}

#[allow(dead_code)]
pub(crate) async fn run_bulk_for_configs_cancellable(
    context: &AppContext,
    settings: ResolvedTestSettings,
    configs: Vec<ConfigRecord>,
    run_kind: &str,
    show_progress: bool,
    cancel_rx: Option<crate::support::cancel::CancellationReceiver>,
) -> crate::app::Result<Vec<TestOutputRow>> {
    bulk_executor::run_bulk_for_configs_cancellable(
        context,
        settings,
        configs,
        run_kind,
        show_progress,
        cancel_rx,
    )
    .await
}

pub(crate) async fn run_bulk_for_configs_with_progress(
    context: &AppContext,
    settings: ResolvedTestSettings,
    configs: Vec<ConfigRecord>,
    run_kind: &str,
    cancel_rx: Option<crate::support::cancel::CancellationReceiver>,
    progress_tx: tokio::sync::mpsc::UnboundedSender<(usize, usize)>,
) -> crate::app::Result<Vec<TestOutputRow>> {
    bulk_executor::run_bulk_for_configs_with_progress(
        context,
        settings,
        configs,
        run_kind,
        cancel_rx,
        progress_tx,
    )
    .await
}
