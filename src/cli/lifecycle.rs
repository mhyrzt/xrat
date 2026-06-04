use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Enable a config.")]
pub struct EnableArgs {
    #[arg(help = "Config ID to enable.")]
    pub id: i64,
}

#[derive(Debug, Args)]
#[command(about = "Disable a config.")]
pub struct DisableArgs {
    #[arg(help = "Config ID to disable.")]
    pub id: i64,
}

#[derive(Debug, Args)]
#[command(about = "Soft delete a config.")]
pub struct DeleteArgs {
    #[arg(help = "Config ID to delete.")]
    pub id: i64,
    #[arg(long = "hard", help = "Permanently delete the config.")]
    pub hard: bool,
}

#[derive(Debug, Args)]
#[command(about = "Restore a soft-deleted config.")]
pub struct RestoreArgs {
    #[arg(help = "Config ID to restore.")]
    pub id: i64,
}

#[derive(Debug, Args)]
#[command(about = "Show details for a config.")]
pub struct ShowArgs {
    #[arg(help = "Config ID to show.")]
    pub id: i64,
    #[arg(long = "json", help = "Print the result as JSON.")]
    pub json: bool,
}
