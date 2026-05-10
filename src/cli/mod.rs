mod add;
mod command;
mod connect;
mod daemon;
mod disconnect;
mod import;
mod list;
mod parse;
mod root;
mod scan;
mod status;
mod test;
mod tests;

pub use add::AddArgs;
pub use command::Command;
pub use connect::ConnectArgs;
pub use daemon::{DaemonAction, DaemonArgs, DaemonStartArgs, DaemonStatusArgs, DaemonStopArgs};
pub use disconnect::DisconnectArgs;
pub use import::ImportArgs;
pub use list::{ListArgs, ListConfigsArgs, ListSubscriptionsArgs, ListTarget, SubscriptionKind};
pub use parse::{ParseArgs, ParseEngine};
pub use root::Cli;
pub use scan::ScanArgs;
pub use status::StatusArgs;
pub use test::{TestArgs, TestFormat, TestSortBy};

use clap::Parser;

pub fn parse() -> Cli {
    Cli::parse()
}
