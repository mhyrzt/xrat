mod add;
mod command;
mod completions;
mod connect;
mod daemon;
mod disconnect;
mod geoip;
mod import;
mod init;
mod lifecycle;
mod list;
mod manpage;
mod parse;
mod proxy;
mod root;
mod scan;
mod serve;
mod status;
mod test_cmd;
mod tests;
mod tui;

pub use add::AddArgs;
pub use command::Command;
pub use completions::CompletionsArgs;
pub use connect::ConnectArgs;
pub use daemon::{
    DaemonAction, DaemonArgs, DaemonInstallArgs, DaemonServeArgs, DaemonStartArgs,
    DaemonStatusArgs, DaemonStopArgs, DaemonUninstallArgs,
};
pub use disconnect::DisconnectArgs;
pub use geoip::{
    GeoIpAction, GeoIpArgs, GeoIpBackendArgs, GeoIpDownloadArgs, GeoIpLookupArgs, GeoIpPathArgs,
    GeoIpStatusArgs, GeoIpUpdateArgs,
};
pub use import::ImportArgs;
pub use init::InitArgs;
pub use lifecycle::{DeleteArgs, DisableArgs, EnableArgs, RestoreArgs, ShowArgs};
pub use list::{ListArgs, ListConfigsArgs, ListSubscriptionsArgs, ListTarget, SubscriptionKind};
pub use manpage::ManpageArgs;
pub use parse::{ParseArgs, ParseEngine};
pub use proxy::{
    ProxyAction, ProxyArgs, ProxyRotateArgs, ProxyStartArgs, ProxyStatusArgs, ProxyStopArgs,
};
pub use root::Cli;
pub use scan::ScanArgs;
pub use serve::ServeArgs;
pub use status::StatusArgs;
pub use test_cmd::{TestArgs, TestFormat, TestSortBy};
pub use tui::TuiArgs;

use clap::Parser;

pub fn parse() -> Cli {
    Cli::try_parse().unwrap_or_else(|e| {
        use clap::error::ErrorKind;
        if matches!(
            e.kind(),
            ErrorKind::MissingRequiredArgument
                | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        ) {
            let mut args: Vec<String> = std::env::args().collect();
            args.push("tui".to_string());
            Cli::parse_from(args)
        } else {
            e.exit()
        }
    })
}
