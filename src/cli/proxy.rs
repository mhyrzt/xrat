use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(about = "Control auto-rotating proxy scheduling via the daemon.")]
pub struct ProxyArgs {
    #[command(subcommand)]
    pub action: ProxyAction,
}

#[derive(Debug, Subcommand)]
pub enum ProxyAction {
    #[command(
        about = "Enable automatic proxy rotation on a fixed schedule. State is volatile and resets to config defaults on daemon restart."
    )]
    Start(ProxyStartArgs),
    #[command(about = "Show the current proxy rotation status.")]
    Status(ProxyStatusArgs),
    #[command(about = "Trigger an immediate manual rotation.")]
    Rotate(ProxyRotateArgs),
    #[command(
        about = "Disable automatic proxy rotation. State is volatile and resets to config defaults on daemon restart."
    )]
    Stop(ProxyStopArgs),
    #[command(about = "Print or locate the Proxy Auto-Config (PAC) file.")]
    Pac(ProxyPacArgs),
}

#[derive(Debug, Args)]
pub struct ProxyPacArgs {
    #[command(subcommand)]
    pub action: ProxyPacAction,
}

#[derive(Debug, Subcommand)]
pub enum ProxyPacAction {
    #[command(about = "Print the PAC URL served by the API server.")]
    Url(ProxyPacUrlArgs),
    #[command(about = "Print the generated PAC file for the active runtime.")]
    Print(ProxyPacPrintArgs),
}

#[derive(Debug, Args, Default)]
pub struct ProxyPacUrlArgs {}

#[derive(Debug, Args, Default)]
pub struct ProxyPacPrintArgs {}

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
    #[arg(
        long = "refresh",
        help = "Refresh URL-backed subscriptions before selecting a candidate."
    )]
    pub refresh: bool,
}

/// No additional arguments.
#[derive(Debug, Args, Default)]
pub struct ProxyStopArgs {}
