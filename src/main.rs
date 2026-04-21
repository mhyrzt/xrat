use xrat::app::{import, path};
use xrat::cli;
use xrat::db::Database;

struct RuntimePaths {
    database_path: std::path::PathBuf,
    config_path: std::path::PathBuf,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::parse();
    let runtime_paths = resolve_runtime_paths(&args)?;
    let (source, nodes) = import::load_nodes(&args.input)?;
    let db = Database::connect(&runtime_paths.database_path).await?;
    let summary = db.import_nodes(&source, &nodes).await?;

    println!(
        "Imported {} parsed nodes into {} using config {} (subscription #{}, total configs: {})",
        summary.imported_configs,
        runtime_paths.database_path.display(),
        runtime_paths.config_path.display(),
        summary.subscription_id,
        summary.total_configs
    );

    Ok(())
}

fn resolve_runtime_paths(args: &cli::Cli) -> Result<RuntimePaths, Box<dyn std::error::Error>> {
    let app_paths = path::ensure_layout()?;
    let database_path = args
        .database
        .clone()
        .unwrap_or_else(|| app_paths.database_path.clone());
    let config_path = args
        .config
        .clone()
        .unwrap_or_else(|| app_paths.config_path.clone());

    path::ensure_config_file(&config_path)?;

    Ok(RuntimePaths {
        database_path,
        config_path,
    })
}
