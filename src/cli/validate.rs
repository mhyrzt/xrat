use std::path::PathBuf;

use clap::{Args, ValueEnum};

#[derive(Debug, Clone, Args)]
#[command(
    about = "Validate an XRAT config.toml file.",
    long_about = "Validate that an XRAT config.toml file exists, parses, and contains internally consistent settings."
)]
pub struct ValidateArgs {
    #[arg(help = "Path to the config.toml file to validate.")]
    pub path: PathBuf,

    #[arg(
        long = "format",
        value_enum,
        default_value_t,
        help = "Output format for the validation result."
    )]
    pub format: ValidateFormat,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum ValidateFormat {
    /// Human-readable output.
    #[default]
    Human,
    /// Machine-readable JSON output.
    Json,
}
