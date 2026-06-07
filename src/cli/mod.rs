mod add;
mod command;
mod completions;
mod connect;
mod daemon;
mod db;
mod disconnect;
mod geoip;
mod import;
mod init;
mod lifecycle;
mod list;
mod logs;
mod manpage;
mod parse;
mod proxy;
mod purge;
mod root;
mod rotate;
mod scan;
mod serve;
mod status;
mod test_cmd;
mod tests;
mod tui;
mod update;
mod upgrade;
mod validate;
mod version;

pub use add::AddArgs;
pub use command::Command;
pub use completions::CompletionsArgs;
pub use connect::ConnectArgs;
pub use daemon::{
    DaemonAction, DaemonArgs, DaemonInstallArgs, DaemonRestartArgs, DaemonServeArgs,
    DaemonStartArgs, DaemonStatusArgs, DaemonStopArgs, DaemonUninstallArgs,
};
pub use db::{DbAction, DbArgs, DbMigrateArgs};
pub use disconnect::DisconnectArgs;
pub use geoip::{
    GeoIpAction, GeoIpArgs, GeoIpBackendArgs, GeoIpDownloadArgs, GeoIpLookupArgs, GeoIpPathArgs,
    GeoIpStatusArgs, GeoIpUpdateArgs,
};
pub use import::ImportArgs;
pub use init::InitArgs;
pub use lifecycle::{
    DeleteArgs, DeleteConfigArgs, DeleteSubscriptionArgs, DeleteTarget, DisableArgs, EnableArgs,
    RestoreArgs, ShowArgs, ShowConfigArgs, ShowSubscriptionArgs, ShowTarget,
};
pub use list::{
    ListArgs, ListConfigsArgs, ListFormat, ListSubscriptionsArgs, ListTarget, SubscriptionKind,
};
pub use logs::{LogLevel, LogSource, LogsArgs, LogsClearArgs, LogsCommand};
pub use manpage::ManpageArgs;
pub use parse::{ParseArgs, ParseEngine};
pub use proxy::{
    ProxyAction, ProxyArgs, ProxyDesktopAction, ProxyDesktopArgs, ProxyDesktopDisableArgs,
    ProxyDesktopEnableArgs, ProxyDesktopKind, ProxyDesktopStatusArgs, ProxyDesktopToggleArgs,
    ProxyInfoArgs, ProxyPacAction, ProxyPacArgs, ProxyPacPrintArgs, ProxyPacUrlArgs,
    ProxyShellAction, ProxyShellArgs, ProxyShellDisableArgs, ProxyShellEnableArgs, ProxyShellKind,
    ProxyShellStatusArgs, ProxyShellToggleArgs,
};
pub use purge::PurgeArgs;
pub use root::Cli;
pub use rotate::{
    RotateAction, RotateArgs, RotateDisableArgs, RotateEnableArgs, RotateNowArgs, RotateStatusArgs,
};
pub use scan::ScanArgs;
pub use serve::ServeArgs;
pub use status::StatusArgs;
pub use test_cmd::{TestArgs, TestFormat, TestSortBy};
pub use tui::TuiArgs;
pub use update::UpdateArgs;
pub use upgrade::UpgradeArgs;
pub use validate::{ValidateArgs, ValidateFormat};
pub use version::VersionArgs;

use clap::Parser;

pub fn parse() -> Cli {
    Cli::try_parse().unwrap_or_else(|e| {
        use clap::error::ErrorKind;
        if matches!(e.kind(), ErrorKind::MissingRequiredArgument) && std::env::args_os().len() == 1
        {
            let mut args: Vec<String> = std::env::args().collect();
            args.push("tui".to_string());
            Cli::parse_from(args)
        } else {
            e.exit()
        }
    })
}
