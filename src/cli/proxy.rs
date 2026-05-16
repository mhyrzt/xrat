use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(about = "Control auto-rotating proxy scheduling via the daemon.")]
pub struct ProxyArgs {
    #[command(subcommand)]
    pub action: ProxyAction,
}

#[derive(Debug, Subcommand)]
pub enum ProxyAction {
    #[command(about = "Enable automatic proxy rotation on a fixed schedule.")]
    Start(ProxyStartArgs),
    #[command(about = "Show the current proxy rotation status.")]
    Status(ProxyStatusArgs),
    #[command(about = "Trigger an immediate manual rotation.")]
    Rotate(ProxyRotateArgs),
    #[command(about = "Disable automatic proxy rotation.")]
    Stop(ProxyStopArgs),
}

/// No additional arguments.
#[derive(Debug, Args, Default)]
pub struct ProxyStartArgs {}

#[derive(Debug, Args, Default)]
pub struct ProxyStatusArgs {
    #[arg(long = "json", help = "Print rotation status as JSON.")]
    pub json: bool,
}

#[derive(Debug, Args, Default)]
pub struct ProxyRotateArgs {
    #[arg(
        long = "config-id",
        help = "Force rotation to a specific enabled config ID."
    )]
    pub config_id: Option<i64>,
}

/// No additional arguments.
#[derive(Debug, Args, Default)]
pub struct ProxyStopArgs {}
