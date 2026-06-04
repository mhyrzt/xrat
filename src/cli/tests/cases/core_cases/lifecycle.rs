use clap::Parser;

use crate::cli::{Cli, Command, DeleteTarget};

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
fn parses_delete_config_subcommand() {
    let cli = Cli::parse_from(["xrat", "delete", "config", "7"]);

    match cli.command {
        Command::Delete(args) => match args.target {
            DeleteTarget::Config(config) => {
                assert_eq!(config.id, 7);
                assert!(!config.hard);
            }
            DeleteTarget::Subscription(_) => panic!("expected config target"),
        },
        _ => panic!("expected delete command"),
    }
}

#[test]
fn parses_delete_config_hard_subcommand() {
    let cli = Cli::parse_from(["xrat", "delete", "config", "--hard", "7"]);

    match cli.command {
        Command::Delete(args) => match args.target {
            DeleteTarget::Config(config) => {
                assert_eq!(config.id, 7);
                assert!(config.hard);
            }
            DeleteTarget::Subscription(_) => panic!("expected config target"),
        },
        _ => panic!("expected delete command"),
    }
}

#[test]
fn parses_delete_subscription_subcommand() {
    let cli = Cli::parse_from(["xrat", "delete", "subscription", "--yes", "3"]);

    match cli.command {
        Command::Delete(args) => match args.target {
            DeleteTarget::Subscription(subscription) => {
                assert_eq!(subscription.id, 3);
                assert!(subscription.yes);
            }
            DeleteTarget::Config(_) => panic!("expected subscription target"),
        },
        _ => panic!("expected delete command"),
    }
}

#[test]
fn delete_without_target_errors() {
    let error = Cli::try_parse_from(["xrat", "delete"]).expect_err("delete target is required");
    assert!(error.to_string().contains("Usage: xrat delete"));
}

#[test]
fn parses_restore_subcommand() {
    let cli = Cli::parse_from(["xrat", "restore", "7"]);

    match cli.command {
        Command::Restore(args) => assert_eq!(args.id, 7),
        _ => panic!("expected restore command"),
    }
}

#[test]
fn parses_purge_subcommand() {
    let cli = Cli::parse_from(["xrat", "purge"]);
    match cli.command {
        Command::Purge(args) => assert!(!args.yes),
        _ => panic!("expected purge command"),
    }

    let cli = Cli::parse_from(["xrat", "purge", "--yes"]);
    match cli.command {
        Command::Purge(args) => assert!(args.yes),
        _ => panic!("expected purge command"),
    }
}
