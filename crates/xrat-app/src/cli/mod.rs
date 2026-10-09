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
mod install;
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
mod setup;
mod status;
mod test_cmd;
mod tests;
mod tui;
mod tun;
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
pub use install::{InstallArgs, InstallCore};
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
    ProxyShellProtocol, ProxyShellStatusArgs, ProxyShellToggleArgs,
};
pub use purge::PurgeArgs;
pub use root::Cli;
pub use rotate::{
    RotateAction, RotateArgs, RotateDisableArgs, RotateEnableArgs, RotateNowArgs, RotateStatusArgs,
};
pub use scan::ScanArgs;
pub use serve::ServeArgs;
pub use setup::{SetupArgs, SetupFormat};
pub use status::StatusArgs;
pub use test_cmd::{TestArgs, TestFormat, TestSortBy};
pub use tui::TuiArgs;
pub use tun::{
    TunAction, TunArgs, TunModeArgs, TunSetSplitModeArgs, TunSetupArgs, TunSplitAction,
    TunSplitAppsArgs, TunSplitArgs, TunSplitClearArgs, TunSplitClearTarget, TunSplitListArgs,
    TunSplitListTarget, TunSplitListViewTarget, TunSplitModeArg, TunSplitModifyArgs, TunStatusArgs,
};
pub use update::UpdateArgs;
pub use upgrade::UpgradeArgs;
pub use validate::{ValidateArgs, ValidateFormat};
pub use version::VersionArgs;

use clap::Parser;

mod parser;
pub use parser::parse;
