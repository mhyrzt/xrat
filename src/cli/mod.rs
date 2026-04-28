mod add;
mod command;
mod import;
mod list;
mod root;
mod test;
mod tests;

pub use add::AddArgs;
pub use command::Command;
pub use import::ImportArgs;
pub use list::{ListArgs, ListConfigsArgs, ListSubscriptionsArgs, ListTarget, SubscriptionKind};
pub use root::Cli;
pub use test::{TestArgs, TestFormat, TestSortBy};

use clap::Parser;

pub fn parse() -> Cli {
    Cli::parse()
}
