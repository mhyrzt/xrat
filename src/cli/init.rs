use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Initialize the xrat config directory, config file, and database.")]
pub struct InitArgs {
    /// Print planned actions without creating any files.
    #[arg(long)]
    pub dry_run: bool,
}
