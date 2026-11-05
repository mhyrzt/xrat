use crate::app::commands::output;
use crate::app::commands::progress::CliProgress;
use crate::app::{context::AppContext, import};

pub async fn run(context: &AppContext, args: &crate::cli::ImportArgs) -> crate::app::Result<()> {
    let progress = CliProgress::spinner(true, "importing subscription");
    let headers = args
        .headers
        .iter()
        .map(|(name, value)| {
            (
                name.to_string(),
                String::from_utf8_lossy(value.as_bytes()).into_owned(),
            )
        })
        .collect();
    let result = if args.url_list {
        import_url_list(context, args).await
    } else {
        crate::app::services::catalog_import::subscription(
            context,
            &args.input,
            args.name.as_deref(),
            &headers,
        )
        .await
    };
    progress.finish_and_clear();
    let summary = result?;
    let subscription = context
        .db
        .get_subscription_by_id(summary.subscription_id)
        .await?;
    let subscription_label = subscription
        .map(|subscription| subscription.r#ref)
        .unwrap_or_else(|| "-".to_string());

    print_summary(
        &summary,
        &subscription_label,
        &context.runtime_paths.database_label,
        &context.runtime_paths.config_path.display().to_string(),
    );
    Ok(())
}

pub(super) fn print_summary(
    summary: &xrat_db::ImportSummary,
    subscription_label: &str,
    database_label: &str,
    config_path: &str,
) {
    println!(
        "{}",
        output::success(
            format!("Imported {} parsed nodes.", summary.imported_configs),
            output::color_enabled()
        )
    );
    println!(
        "{}",
        output::format_kv(
            None,
            &[
                ("database", database_label.to_string()),
                ("config", config_path.to_string()),
                ("subscription", subscription_label.to_string()),
                ("removed configs", summary.removed_configs.to_string()),
                ("total configs", summary.total_configs.to_string()),
            ],
            output::color_enabled(),
        )
    );
}

async fn import_url_list(
    context: &AppContext,
    args: &crate::cli::ImportArgs,
) -> crate::app::Result<xrat_db::ImportSummary> {
    let (source, nodes) = import::load_url_list(&args.input).await?;
    import::persist_nodes(&context.db, source, &nodes, args.name.as_deref(), None).await
}
