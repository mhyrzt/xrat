use crate::app::commands::output;
use crate::app::commands::progress::CliProgress;
use crate::app::{context::AppContext, import};

pub async fn run(context: &AppContext, input: &str, name: Option<&str>) -> crate::app::Result<()> {
    let progress = CliProgress::spinner(true, "importing subscription");
    let result = async {
        let (source, nodes) = import::load_nodes_async(input).await?;
        let summary = import::persist_nodes(&context.db, source, &nodes, name).await?;
        crate::app::Result::Ok(summary)
    }
    .await;
    progress.finish_and_clear();
    let summary = result?;
    let subscription = context
        .db
        .get_subscription_by_id(summary.subscription_id)
        .await?;
    let subscription_label = subscription
        .map(|subscription| subscription.r#ref)
        .unwrap_or_else(|| "-".to_string());

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
                ("database", context.runtime_paths.database_label.clone()),
                (
                    "config",
                    context.runtime_paths.config_path.display().to_string()
                ),
                ("subscription", subscription_label),
                ("removed configs", summary.removed_configs.to_string()),
                ("total configs", summary.total_configs.to_string()),
            ],
            output::color_enabled(),
        )
    );

    Ok(())
}
