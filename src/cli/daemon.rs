use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(about = "Run or control the XRAT daemon supervisor process.")]
pub struct DaemonArgs {
    #[command(subcommand)]
    pub action: DaemonAction,
}

#[derive(Debug, Subcommand)]
pub enum DaemonAction {
    #[command(about = "Start the long-lived XRAT daemon process.")]
    Start(DaemonStartArgs),
    #[command(hide = true, about = "Internal: run the daemon IPC server loop.")]
    RunServer(DaemonServeArgs),
    #[command(about = "Show daemon IPC reachability and protocol information.")]
    Status(DaemonStatusArgs),
    #[command(about = "Request daemon shutdown via local IPC.")]
    Stop(DaemonStopArgs),
}

/// No additional arguments.
#[derive(Debug, Args, Default)]
pub struct DaemonStartArgs {}

/// No additional arguments.
#[derive(Debug, Args, Default)]
pub struct DaemonServeArgs {}

/// No additional arguments.
#[derive(Debug, Args, Default)]
pub struct DaemonStatusArgs {}

/// No additional arguments.
#[derive(Debug, Args, Default)]
pub struct DaemonStopArgs {}
