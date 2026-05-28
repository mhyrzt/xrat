use clap::Subcommand;

use crate::cli::{
    AddArgs, ConnectArgs, DaemonArgs, DisconnectArgs, ImportArgs, ListArgs, ParseArgs, ProxyArgs,
    ScanArgs, ServeArgs, StatusArgs, TestArgs, TuiArgs,
};

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Import a subscription URL, file, or raw text into SQLite.")]
    Import(ImportArgs),
    #[command(about = "Add a single config URI directly to SQLite.")]
    Add(AddArgs),
    #[command(about = "List stored configs or subscription sources.")]
    List(ListArgs),
    #[command(about = "Test connectivity and latency for stored configs.")]
    Test(Box<TestArgs>),
    #[command(about = "Scan candidate IPs for TCP reachability and persist results.")]
    Scan(ScanArgs),
    #[command(about = "Start a managed proxy runtime for a stored config.")]
    Connect(ConnectArgs),
    #[command(about = "Stop the active managed proxy runtime.")]
    Disconnect(DisconnectArgs),
    #[command(about = "Show the managed proxy runtime status.")]
    Status(StatusArgs),
    #[command(about = "Run or control the XRAT daemon supervisor process.")]
    Daemon(DaemonArgs),
    #[command(about = "Control auto-rotating proxy scheduling via the daemon.")]
    Proxy(ProxyArgs),
    #[command(about = "Start the local Axum HTTP API server.")]
    Serve(ServeArgs),
    #[command(about = "Start the interactive terminal UI.")]
    Tui(TuiArgs),
    #[command(about = "Parse and validate config links without persisting.")]
    Parse(ParseArgs),
}
