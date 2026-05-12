use clap::{CommandFactory, Parser};

use crate::cli::{Cli, Command, ProxyAction};

#[test]
fn parses_proxy_subcommands() {
    let start = Cli::parse_from(["xrat", "proxy", "start"]);
    match start.command {
        Command::Proxy(args) => assert!(matches!(args.action, ProxyAction::Start(_))),
        _ => panic!("expected proxy command"),
    }

    let status = Cli::parse_from(["xrat", "proxy", "status"]);
    match status.command {
        Command::Proxy(args) => assert!(matches!(args.action, ProxyAction::Status(_))),
        _ => panic!("expected proxy command"),
    }

    let rotate = Cli::parse_from(["xrat", "proxy", "rotate", "--config-id", "42"]);
    match rotate.command {
        Command::Proxy(args) => match args.action {
            ProxyAction::Rotate(rotate_args) => assert_eq!(rotate_args.config_id, Some(42)),
            _ => panic!("expected rotate subcommand"),
        },
        _ => panic!("expected proxy command"),
    }

    let stop = Cli::parse_from(["xrat", "proxy", "stop"]);
    match stop.command {
        Command::Proxy(args) => assert!(matches!(args.action, ProxyAction::Stop(_))),
        _ => panic!("expected proxy command"),
    }
}

#[test]
fn proxy_rotate_help_mentions_config_flag() {
    let mut cmd = Cli::command();
    let proxy_cmd = cmd
        .find_subcommand_mut("proxy")
        .expect("proxy subcommand should exist");
    let rotate_cmd = proxy_cmd
        .find_subcommand_mut("rotate")
        .expect("proxy rotate subcommand should exist");
    let has_config = rotate_cmd
        .get_arguments()
        .any(|arg| arg.get_long() == Some("config-id"));
    assert!(has_config, "proxy rotate should expose --config-id");
}
