use clap::Subcommand;

use crate::cli::{AddArgs, ImportArgs, ListArgs, TestArgs};

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Import a subscription source or batch of configs into SQLite.")]
    Import(ImportArgs),
    #[command(about = "Add one config URI directly into SQLite.")]
    Add(AddArgs),
    #[command(about = "List persisted nodes or subscriptions.")]
    List(ListArgs),
    #[command(about = "Test connectivity and latency for a stored config.")]
    Test(TestArgs),
}
