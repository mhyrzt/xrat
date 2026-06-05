use clap::Subcommand;

use crate::cli::{
    AddArgs, CompletionsArgs, ConnectArgs, DaemonArgs, DeleteArgs, DisableArgs, DisconnectArgs,
    EnableArgs, GeoIpArgs, ImportArgs, InitArgs, ListArgs, LogsArgs, ManpageArgs, ParseArgs,
    ProxyArgs, PurgeArgs, RestoreArgs, ScanArgs, ServeArgs, ShowArgs, StatusArgs, TestArgs,
    TuiArgs, UpgradeArgs, ValidateArgs, VersionArgs,
};

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Initialize config directory, config file, and database.")]
    Init(InitArgs),
    #[command(about = "Import a subscription URL, file, or raw text into SQLite.")]
    Import(ImportArgs),
    #[command(about = "Add a single config URI directly to SQLite.")]
    Add(AddArgs),
    #[command(about = "List stored configs or subscription sources.")]
    List(ListArgs),
    #[command(about = "Show details for a config or subscription.")]
    Show(ShowArgs),
    #[command(about = "Enable a config.")]
    Enable(EnableArgs),
    #[command(about = "Disable a config.")]
    Disable(DisableArgs),
    #[command(about = "Delete a config (soft/hard) or a subscription with its configs.")]
    Delete(DeleteArgs),
    #[command(about = "Restore a soft-deleted config.")]
    Restore(RestoreArgs),
    #[command(about = "Permanently delete all soft-deleted configs.")]
    Purge(PurgeArgs),
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
    #[command(about = "Show app events plus xray-core / sing-box engine logs.")]
    Logs(LogsArgs),
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
    #[command(about = "Validate an XRAT config.toml file.")]
    Validate(ValidateArgs),
    #[command(about = "Self-upgrade xrat from the latest release or by building from source.")]
    Upgrade(UpgradeArgs),
    #[command(about = "Print the xrat version.")]
    Version(VersionArgs),
    #[command(name = "geoip", about = "Inspect and manage GeoLite2 MMDB assets.")]
    GeoIp(GeoIpArgs),
    #[command(hide = true)]
    Manpage(ManpageArgs),
    #[command(hide = true)]
    Completions(CompletionsArgs),
}
