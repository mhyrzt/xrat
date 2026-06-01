use clap::Parser;

use crate::cli::{Cli, Command};

#[test]
fn parses_select_subcommand() {
    let cli = Cli::parse_from(["xrat", "select", "42"]);

    match cli.command {
        Command::Select(args) => assert_eq!(args.id, 42),
        _ => panic!("expected select command"),
    }
}

#[test]
fn parses_enable_subcommand() {
    let cli = Cli::parse_from(["xrat", "enable", "7"]);

    match cli.command {
        Command::Enable(args) => assert_eq!(args.id, 7),
        _ => panic!("expected enable command"),
    }
}

#[test]
fn parses_disable_subcommand() {
    let cli = Cli::parse_from(["xrat", "disable", "7"]);

    match cli.command {
        Command::Disable(args) => assert_eq!(args.id, 7),
        _ => panic!("expected disable command"),
    }
}

#[test]
fn parses_delete_subcommand() {
    let cli = Cli::parse_from(["xrat", "delete", "7"]);

    match cli.command {
        Command::Delete(args) => {
            assert_eq!(args.id, 7);
            assert!(!args.hard);
        }
        _ => panic!("expected delete command"),
    }
}

#[test]
fn parses_delete_hard_subcommand() {
    let cli = Cli::parse_from(["xrat", "delete", "--hard", "7"]);

    match cli.command {
        Command::Delete(args) => {
            assert_eq!(args.id, 7);
            assert!(args.hard);
        }
        _ => panic!("expected delete command"),
    }
}

#[test]
fn parses_restore_subcommand() {
    let cli = Cli::parse_from(["xrat", "restore", "7"]);

    match cli.command {
        Command::Restore(args) => assert_eq!(args.id, 7),
        _ => panic!("expected restore command"),
    }
}
