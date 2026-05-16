use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct DaemonArgs {
    #[command(subcommand)]
    pub action: DaemonAction,
}

#[derive(Debug, Subcommand)]
pub enum DaemonAction {
    #[command(about = "Start the long-lived XRAT daemon process.")]
    Start(DaemonStartArgs),
    #[command(hide = true)]
    RunServer(DaemonServeArgs),
    #[command(about = "Show daemon IPC reachability and protocol information.")]
    Status(DaemonStatusArgs),
    #[command(about = "Request daemon shutdown via local IPC.")]
    Stop(DaemonStopArgs),
}

#[derive(Debug, Args, Default)]
pub struct DaemonStartArgs {}

#[derive(Debug, Args, Default)]
pub struct DaemonServeArgs {}

#[derive(Debug, Args, Default)]
pub struct DaemonStatusArgs {}

#[derive(Debug, Args, Default)]
pub struct DaemonStopArgs {}
