use clap::Args;

#[derive(Debug, Args, Default)]
#[command(about = "Permanently delete all soft-deleted configs.")]
pub struct PurgeArgs {
    #[arg(long = "yes", help = "Skip the confirmation prompt.")]
    pub yes: bool,
}
