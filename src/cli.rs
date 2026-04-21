use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(about = "Read a subscription source and persist parsed configs into SQLite.")]
pub struct Cli {
    pub input: String,
    #[arg(long)]
    pub database_path: Option<PathBuf>,
}

pub fn parse() -> Cli {
    Cli::parse()
}
