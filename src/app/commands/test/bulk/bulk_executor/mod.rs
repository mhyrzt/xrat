use super::super::*;

mod bulk;
mod progress;
mod single;

pub(super) async fn run_single(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
    config_id: i64,
) -> crate::app::Result<()> {
    single::run_single(args, context, settings, config_id).await
}

pub(super) async fn run_bulk(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
) -> crate::app::Result<()> {
    bulk::run_bulk(args, context, settings).await
}

pub(crate) async fn run_bulk_for_configs(
    context: &AppContext,
    settings: ResolvedTestSettings,
    configs: Vec<ConfigRecord>,
    run_kind: &str,
    show_progress: bool,
) -> crate::app::Result<Vec<TestOutputRow>> {
    bulk::run_bulk_for_configs(context, settings, configs, run_kind, show_progress).await
}

pub(crate) async fn run_bulk_for_configs_cancellable(
    context: &AppContext,
    settings: ResolvedTestSettings,
    configs: Vec<ConfigRecord>,
    run_kind: &str,
    show_progress: bool,
    cancel_rx: Option<crate::support::cancel::CancellationReceiver>,
) -> crate::app::Result<Vec<TestOutputRow>> {
    bulk::run_bulk_for_configs_cancellable(
        context,
        settings,
        configs,
        run_kind,
        show_progress,
        cancel_rx,
        None,
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
    bulk::run_bulk_for_configs_cancellable(
        context,
        settings,
        configs,
        run_kind,
        false,
        cancel_rx,
        Some(progress_tx),
    )
    .await
}
