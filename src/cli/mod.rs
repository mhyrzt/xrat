mod add;
mod command;
mod connect;
mod daemon;
mod disconnect;
mod geoip;
mod import;
mod lifecycle;
mod list;
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
pub use connect::ConnectArgs;
pub use daemon::{
    DaemonAction, DaemonArgs, DaemonServeArgs, DaemonStartArgs, DaemonStatusArgs, DaemonStopArgs,
};
pub use disconnect::DisconnectArgs;
pub use geoip::{GeoIpAction, GeoIpArgs, GeoIpPathArgs, GeoIpStatusArgs};
pub use import::ImportArgs;
pub use lifecycle::{DeleteArgs, DisableArgs, EnableArgs, RestoreArgs, SelectArgs, ShowArgs};
pub use list::{ListArgs, ListConfigsArgs, ListSubscriptionsArgs, ListTarget, SubscriptionKind};
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
    Cli::parse()
}
