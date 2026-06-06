use clap::Parser;

use crate::cli::{Cli, Command, RotateAction};

#[test]
fn parses_rotate_subcommands() {
    let start = Cli::parse_from(["xrat", "rotate", "start"]);
    match start.command {
        Command::Rotate(args) => assert!(matches!(args.action, RotateAction::Start(_))),
        _ => panic!("expected rotate command"),
    }

    let stop = Cli::parse_from(["xrat", "rotate", "stop"]);
    match stop.command {
        Command::Rotate(args) => assert!(matches!(args.action, RotateAction::Stop(_))),
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
