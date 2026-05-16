use std::path::PathBuf;

use clap::Parser;

use super::paths::resolve_runtime;
use crate::cli::Cli;

mod binary_cases;
mod database_cases;

fn temp_root(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{prefix}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should be valid")
            .as_nanos()
    ))
}

fn cli_for_config(config_path: &std::path::Path) -> Cli {
    Cli::parse_from([
        "xrat",
        "--config",
        config_path.to_str().unwrap(),
        "list",
        "configs",
    ])
}
