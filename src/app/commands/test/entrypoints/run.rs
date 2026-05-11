use super::super::*;
use super::{print_latest_run_summary, run_ping_loop};

pub async fn run(args: &TestArgs, context: &AppContext) -> crate::app::Result<()> {
    if args.latest_run_summary {
        print_latest_run_summary(&context.db, args).await?;
        return Ok(());
    }

    let settings = resolve_test_settings(args, &context.app_config, &context.runtime_paths)?;

    if args.ping {
        return run_ping_loop(args, context, settings).await;
    }

    if let Some(config_id) = args.id {
        run_single(args, context, settings, config_id).await
    } else {
        run_bulk(args, context, settings).await
    }
}
