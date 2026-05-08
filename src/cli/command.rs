use clap::Subcommand;

use crate::cli::{
    AddArgs, ConnectArgs, DisconnectArgs, ImportArgs, ListArgs, ParseArgs, StatusArgs, TestArgs,
};

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Import a subscription source or batch of configs into SQLite.")]
    Import(ImportArgs),
    #[command(about = "Add one config URI directly into SQLite.")]
    Add(AddArgs),
    #[command(about = "List persisted nodes or subscriptions.")]
    List(ListArgs),
    #[command(about = "Test connectivity and latency for stored configs.")]
    Test(TestArgs),
    #[command(about = "Start a managed Xray runtime for a stored config.")]
    Connect(ConnectArgs),
    #[command(about = "Stop the active managed Xray runtime.")]
    Disconnect(DisconnectArgs),
    #[command(about = "Show the managed Xray runtime status.")]
    Status(StatusArgs),
    #[command(about = "Parse and validate config links without importing to the database.")]
    Parse(ParseArgs),
}
