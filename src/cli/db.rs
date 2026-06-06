use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(about = "Inspect and maintain the XRAT database.")]
pub struct DbArgs {
    #[command(subcommand)]
    pub action: DbAction,
}

#[derive(Debug, Subcommand)]
pub enum DbAction {
    #[command(about = "Apply any pending database migrations and report the result.")]
    Migrate(DbMigrateArgs),
}

#[derive(Debug, Args, Default)]
pub struct DbMigrateArgs {}
