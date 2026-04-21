use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    about = "Read a subscription source from a URL or file and parse configs into JSON."
)]
pub struct Cli {
    pub input: String,
    pub output_file: PathBuf,
}

pub fn parse() -> Cli {
    Cli::parse()
}
