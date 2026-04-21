use crate::app::{import, runtime::AppContext};

pub async fn run(context: &AppContext, input: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (source, node) = import::load_single_node(input)?;
    let summary = context.db.import_nodes(&source, &[node]).await?;

    println!(
        "Added 1 config into {} using config {} (source: {}, total configs: {})",
        context.runtime_paths.database_path.display(),
        context.runtime_paths.config_path.display(),
        source.kind.as_str(),
        summary.total_configs
    );

    Ok(())
}
