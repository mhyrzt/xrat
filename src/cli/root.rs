use std::path::PathBuf;

use clap::Parser;

use crate::cli::Command;

#[derive(Debug, Parser)]
#[command(about = "Manage XRAT configs and persisted app state.")]
pub struct Cli {
    #[arg(
        long = "database",
        global = true,
        help = "Override the SQLite database path. Defaults to XRAT_PATH/db.sqlite or $HOME/.config/xrat/db.sqlite."
    )]
    pub database: Option<PathBuf>,
    #[arg(
        long = "config",
        global = true,
        help = "Override the config file path. Defaults to XRAT_PATH/Config.toml or $HOME/.config/xrat/Config.toml."
    )]
    pub config: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}
