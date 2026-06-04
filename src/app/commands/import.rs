use crate::app::commands::output;
use crate::app::{context::AppContext, import};

pub async fn run(context: &AppContext, input: &str) -> crate::app::Result<()> {
    let (source, nodes) = import::load_nodes_async(input).await?;
    let summary = context.db.import_nodes(&source, &nodes).await?;

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
                ("subscription", format!("#{}", summary.subscription_id)),
                ("total configs", summary.total_configs.to_string()),
            ],
            output::color_enabled(),
        )
    );

    Ok(())
}
