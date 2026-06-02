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
    #[command(about = "Install xrat-daemon.service as a systemd user service.")]
    Install(DaemonInstallArgs),
    #[command(about = "Remove the installed xrat-daemon.service systemd user service.")]
    Uninstall(DaemonUninstallArgs),
}

#[derive(Debug, Args, Default)]
pub struct DaemonStartArgs {}

#[derive(Debug, Args, Default)]
pub struct DaemonServeArgs {}

#[derive(Debug, Args, Default)]
pub struct DaemonStatusArgs {}

#[derive(Debug, Args, Default)]
pub struct DaemonStopArgs {}

#[derive(Debug, Args)]
pub struct DaemonInstallArgs {
    /// Start the daemon immediately after enabling.
    #[arg(long)]
    pub start: bool,

    /// Print the generated service unit without writing files.
    #[arg(long)]
    pub dry_run: bool,

    /// Install the API server service alongside the daemon.
    #[arg(long)]
    pub with_api: bool,
}

#[derive(Debug, Args)]
pub struct DaemonUninstallArgs {
    /// Print actions without executing them.
    #[arg(long)]
    pub dry_run: bool,
}
