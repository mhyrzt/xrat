use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(about = "Read a subscription source and persist parsed configs into SQLite.")]
pub struct Cli {
    #[arg(help = "Subscription input: URL, file path, or raw subscription text.")]
    pub input: String,
    #[arg(
        long = "database",
        help = "Override the SQLite database path. Defaults to XRAT_PATH/db.sqlite or $HOME/.config/xrat/db.sqlite."
    )]
    pub database: Option<PathBuf>,
    #[arg(
        long = "config",
        help = "Override the config file path. Defaults to XRAT_PATH/Config.toml or $HOME/.config/xrat/Config.toml."
    )]
    pub config: Option<PathBuf>,
}

pub fn parse() -> Cli {
    Cli::parse()
}
