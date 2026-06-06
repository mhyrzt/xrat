use super::*;

mod ping;
mod run;
mod summary;

pub use run::run;

pub(super) async fn run_ping_loop(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
    config_id: i64,
) -> crate::app::Result<()> {
    ping::run_ping_loop(args, context, settings, config_id).await
}

pub(super) async fn print_latest_run_summary(
    db: &Database,
    args: &TestArgs,
) -> crate::app::Result<()> {
    summary::print_latest_run_summary(db, args).await
}

#[cfg(test)]
pub(crate) fn filter_latest_run_rows(
    rows: Vec<crate::db::ConnectionTestRecord>,
    country: Option<&str>,
    asn: Option<&str>,
) -> Vec<crate::db::ConnectionTestRecord> {
    summary::filter_latest_run_rows(rows, country, asn)
}
