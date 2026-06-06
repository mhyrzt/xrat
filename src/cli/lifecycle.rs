use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(about = "Enable a config.")]
pub struct EnableArgs {
    #[arg(help = "Config ID or ref prefix to enable.")]
    pub id: String,
}

#[derive(Debug, Args)]
#[command(about = "Disable a config.")]
pub struct DisableArgs {
    #[arg(help = "Config ID or ref prefix to disable.")]
    pub id: String,
}

#[derive(Debug, Args)]
#[command(about = "Restore a soft-deleted config.")]
pub struct RestoreArgs {
    #[arg(help = "Config ID or ref prefix to restore.")]
    pub id: String,
}

#[derive(Debug, Args)]
#[command(about = "Soft delete a config or delete a subscription and its configs.")]
pub struct DeleteArgs {
    #[command(subcommand)]
    pub target: DeleteTarget,
}

#[derive(Debug, Subcommand)]
pub enum DeleteTarget {
    #[command(about = "Soft delete a config (use --hard to delete permanently).")]
    Config(DeleteConfigArgs),
    #[command(about = "Delete a subscription and all of its configs.")]
    Subscription(DeleteSubscriptionArgs),
}

#[derive(Debug, Args)]
pub struct DeleteConfigArgs {
    #[arg(help = "Config ID or ref prefix to delete.")]
    pub id: String,
    #[arg(long = "hard", help = "Permanently delete the config.")]
    pub hard: bool,
}

#[derive(Debug, Args)]
pub struct DeleteSubscriptionArgs {
    #[arg(help = "Subscription ID or ref prefix to delete.")]
    pub id: String,
    #[arg(long = "yes", help = "Skip the confirmation prompt.")]
    pub yes: bool,
}

#[derive(Debug, Args)]
#[command(about = "Show details for a config or subscription.")]
pub struct ShowArgs {
    #[command(subcommand)]
    pub target: ShowTarget,
}

#[derive(Debug, Subcommand)]
pub enum ShowTarget {
    #[command(about = "Show details for a config.")]
    Config(ShowConfigArgs),
    #[command(about = "Show details for a subscription.")]
    Subscription(ShowSubscriptionArgs),
}

#[derive(Debug, Args)]
pub struct ShowConfigArgs {
    #[arg(help = "Config ID or ref prefix to show.")]
    pub id: String,
    #[arg(long = "json", help = "Print the result as JSON.")]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct ShowSubscriptionArgs {
    #[arg(help = "Subscription ID or ref prefix to show.")]
    pub id: String,
    #[arg(long = "json", help = "Print the result as JSON.")]
    pub json: bool,
}
