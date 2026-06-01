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
