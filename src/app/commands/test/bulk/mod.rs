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
