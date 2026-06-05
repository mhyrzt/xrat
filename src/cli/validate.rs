use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Clone, Args)]
#[command(
    about = "Validate an XRAT config.toml file.",
    long_about = "Validate that an XRAT config.toml file exists, parses, and contains internally consistent settings."
)]
pub struct ValidateArgs {
    #[arg(help = "Path to the config.toml file to validate.")]
    pub path: PathBuf,
}
