use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(about = "Control automatic proxy rotation scheduling via the daemon.")]
pub struct RotateArgs {
    #[command(subcommand)]
    pub action: RotateAction,
}

#[derive(Debug, Subcommand)]
pub enum RotateAction {
    #[command(
        about = "Enable automatic proxy rotation on a fixed schedule. State is volatile and resets to config defaults on daemon restart."
    )]
    Enable(RotateEnableArgs),
    #[command(
        about = "Disable automatic proxy rotation. State is volatile and resets to config defaults on daemon restart."
    )]
    Disable(RotateDisableArgs),
    #[command(about = "Show the current proxy rotation status.")]
    Status(RotateStatusArgs),
    #[command(about = "Trigger an immediate manual rotation.")]
    Now(RotateNowArgs),
}

/// No additional arguments.
#[derive(Debug, Args, Default)]
pub struct RotateEnableArgs {}

/// No additional arguments.
#[derive(Debug, Args, Default)]
pub struct RotateDisableArgs {}

#[derive(Debug, Args, Default)]
pub struct RotateStatusArgs {
    #[arg(long = "json", help = "Print rotation status as JSON.")]
    pub json: bool,
}

#[derive(Debug, Args, Default)]
pub struct RotateNowArgs {
    #[arg(
        long = "config-id",
        help = "Force rotation to a specific enabled config ID or ref prefix."
    )]
    pub config_id: Option<String>,
    #[arg(
        long = "refresh",
        help = "Refresh URL-backed subscriptions before selecting a candidate."
    )]
    pub refresh: bool,
}
