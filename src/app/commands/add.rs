use crate::app::commands::output;
use crate::app::commands::progress::CliProgress;
use crate::app::{context::AppContext, import};

pub async fn run(context: &AppContext, input: &str) -> crate::app::Result<()> {
    let progress = CliProgress::spinner(true, "adding config");
    let result = async {
        let (source, node) = import::load_single_node(input)?;
        let summary = context.db.import_nodes(&source, &[node]).await?;
        crate::app::Result::Ok((source, summary))
    }
    .await;
    progress.finish_and_clear();
    let (source, summary) = result?;

    println!(
        "{}",
        output::success("Added 1 config.", output::color_enabled())
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
                ("source", source.kind.as_str().to_string()),
                ("total configs", summary.total_configs.to_string()),
            ],
            output::color_enabled(),
        )
    );

    Ok(())
}
