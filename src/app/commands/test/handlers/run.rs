use super::super::*;
use super::{print_latest_run_summary, run_ping_loop};
use crate::app::commands::resolve::{resolve_config_id, resolve_subscription_id};

pub async fn run(args: &TestArgs, context: &AppContext) -> crate::app::Result<()> {
    if args.latest_run_summary {
        print_latest_run_summary(&context.db, args).await?;
        return Ok(());
    }

    let settings = resolve_test_settings(args, &context.app_config, &context.runtime_paths)?;
    let config_id = match args.id.as_deref() {
        Some(raw) => Some(resolve_config_id(context, raw).await?),
        None => None,
    };
    let subscription_id = match args.subscription.as_deref() {
        Some(raw) => Some(resolve_subscription_id(context, raw).await?),
        None => None,
    };

    if args.ping {
        let config_id = config_id.ok_or_else(|| {
            AppError::InvalidArgument(
                "`test --ping` requires config id or ref: `xrat test <id-or-ref> --ping`".into(),
            )
        })?;
        return run_ping_loop(args, context, settings, config_id).await;
    }

    if let Some(config_id) = config_id {
        run_single(args, context, settings, config_id).await
    } else {
        run_bulk(args, context, settings, subscription_id).await
    }
}
