use clap::Parser;

use crate::cli::{Cli, Command, RotateAction};

#[test]
fn parses_rotate_subcommands() {
    let enable = Cli::parse_from(["xrat", "rotate", "enable"]);
    match enable.command {
        Command::Rotate(args) => assert!(matches!(args.action, RotateAction::Enable(_))),
        _ => panic!("expected rotate command"),
    }

    let disable = Cli::parse_from(["xrat", "rotate", "disable"]);
    match disable.command {
        Command::Rotate(args) => assert!(matches!(args.action, RotateAction::Disable(_))),
        _ => panic!("expected rotate command"),
    }

    let status = Cli::parse_from(["xrat", "rotate", "status", "--json"]);
    match status.command {
        Command::Rotate(args) => match args.action {
            RotateAction::Status(status_args) => assert!(status_args.json),
            _ => panic!("expected status subcommand"),
        },
        _ => panic!("expected rotate command"),
    }
}

#[test]
fn old_rotate_subcommands_are_removed() {
    for removed in [["xrat", "rotate", "start"], ["xrat", "rotate", "stop"]] {
        assert!(
            Cli::try_parse_from(removed).is_err(),
            "{removed:?} should no longer parse"
        );
    }
}

#[test]
fn parses_rotate_now_flags() {
    let cli = Cli::parse_from(["xrat", "rotate", "now", "--config-id", "42", "--refresh"]);
    match cli.command {
        Command::Rotate(args) => match args.action {
            RotateAction::Now(now) => {
                assert_eq!(now.config_id.as_deref(), Some("42"));
                assert!(now.refresh);
            }
            _ => panic!("expected now subcommand"),
        },
        _ => panic!("expected rotate command"),
    }
}

#[test]
fn parses_rotate_now_config_ref_prefix() {
    let cli = Cli::parse_from(["xrat", "rotate", "now", "--config-id", "a1b2"]);
    match cli.command {
        Command::Rotate(args) => match args.action {
            RotateAction::Now(now) => assert_eq!(now.config_id.as_deref(), Some("a1b2")),
            _ => panic!("expected now subcommand"),
        },
        _ => panic!("expected rotate command"),
    }
}
