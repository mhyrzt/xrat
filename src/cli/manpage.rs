use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Generate man pages for xrat and all subcommands.")]
pub struct ManpageArgs {
    /// Directory to write generated man pages into.
    #[arg(long, default_value = ".")]
    pub output: PathBuf,
}
