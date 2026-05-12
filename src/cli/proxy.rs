use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct ProxyArgs {
    #[command(subcommand)]
    pub action: ProxyAction,
}

#[derive(Debug, Subcommand)]
pub enum ProxyAction {
    #[command(about = "Enable daemon-owned proxy rotation scheduling.")]
    Start(ProxyStartArgs),
    #[command(about = "Show daemon-owned proxy rotation status.")]
    Status(ProxyStatusArgs),
    #[command(about = "Perform a manual rotation, optionally forcing one config id.")]
    Rotate(ProxyRotateArgs),
    #[command(about = "Disable daemon-owned proxy rotation scheduling.")]
    Stop(ProxyStopArgs),
}

#[derive(Debug, Args, Default)]
pub struct ProxyStartArgs {}

#[derive(Debug, Args, Default)]
pub struct ProxyStatusArgs {
    #[arg(long = "json", help = "Print proxy rotation status as JSON.")]
    pub json: bool,
}

#[derive(Debug, Args, Default)]
pub struct ProxyRotateArgs {
    #[arg(
        long = "config-id",
        help = "Target a specific enabled config id for this rotation."
    )]
    pub config_id: Option<i64>,
}

#[derive(Debug, Args, Default)]
pub struct ProxyStopArgs {}
