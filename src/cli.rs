use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(about = "Read a subscription source and persist parsed configs into SQLite.")]
pub struct Cli {
    pub input: String,
    pub database_path: PathBuf,
}

pub fn parse() -> Cli {
    Cli::parse()
}
