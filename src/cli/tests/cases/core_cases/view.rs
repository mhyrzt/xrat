use clap::Parser;

use crate::cli::{Cli, Command, ShowTarget};

#[test]
fn parses_serve_overrides() {
    let cli = Cli::parse_from(["xrat", "serve", "--host", "0.0.0.0", "--port", "9090"]);

    match cli.command {
        Command::Serve(args) => {
            assert_eq!(args.host.as_deref(), Some("0.0.0.0"));
            assert_eq!(args.port, Some(9090));
        }
        _ => panic!("expected serve command"),
    }
}

#[test]
fn parses_tui_subcommand() {
    let cli = Cli::parse_from(["xrat", "tui"]);

    match cli.command {
        Command::Tui(_) => {}
        _ => panic!("expected tui command"),
    }
}

#[test]
fn parses_show_config_subcommand() {
    let cli = Cli::parse_from(["xrat", "show", "config", "7"]);

    match cli.command {
        Command::Show(args) => match args.target {
            ShowTarget::Config(config) => {
                assert_eq!(config.id, 7);
                assert!(!config.json);
            }
            ShowTarget::Subscription(_) => panic!("expected config target"),
        },
        _ => panic!("expected show command"),
    }
}

#[test]
fn parses_show_config_json_subcommand() {
    let cli = Cli::parse_from(["xrat", "show", "config", "--json", "7"]);

    match cli.command {
        Command::Show(args) => match args.target {
            ShowTarget::Config(config) => {
                assert_eq!(config.id, 7);
                assert!(config.json);
            }
            ShowTarget::Subscription(_) => panic!("expected config target"),
        },
        _ => panic!("expected show command"),
    }
}

#[test]
fn parses_show_subscription_json_subcommand() {
    let cli = Cli::parse_from(["xrat", "show", "subscription", "--json", "4"]);

    match cli.command {
        Command::Show(args) => match args.target {
            ShowTarget::Subscription(subscription) => {
                assert_eq!(subscription.id, 4);
                assert!(subscription.json);
            }
            ShowTarget::Config(_) => panic!("expected subscription target"),
        },
        _ => panic!("expected show command"),
    }
}
