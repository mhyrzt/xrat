use clap::Parser;

use crate::cli::{Cli, Command, DbAction};

#[test]
fn parses_db_migrate() {
    let cli = Cli::parse_from(["xrat", "db", "migrate"]);
    match cli.command {
        Command::Db(args) => assert!(matches!(args.action, DbAction::Migrate(_))),
        _ => panic!("expected db command"),
    }
}

#[test]
fn db_requires_subcommand() {
    let error = Cli::try_parse_from(["xrat", "db"]).expect_err("db requires a subcommand");
    let _ = error;
}
