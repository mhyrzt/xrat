use crate::app::{import, runtime::AppContext};

pub async fn run(context: &AppContext, input: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (source, nodes) = import::load_nodes(input)?;
    let summary = context.db.import_nodes(&source, &nodes).await?;

    println!(
        "Imported {} parsed nodes into {} using config {} (subscription #{}, total configs: {})",
        summary.imported_configs,
        context.runtime_paths.database_label,
        context.runtime_paths.config_path.display(),
        summary.subscription_id,
        summary.total_configs
    );

    Ok(())
}
