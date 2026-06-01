use clap::Parser;

use crate::cli::{Cli, Command};

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
fn parses_show_subcommand() {
    let cli = Cli::parse_from(["xrat", "show", "7"]);

    match cli.command {
        Command::Show(args) => {
            assert_eq!(args.id, 7);
            assert!(!args.json);
        }
        _ => panic!("expected show command"),
    }
}

#[test]
fn parses_show_json_subcommand() {
    let cli = Cli::parse_from(["xrat", "show", "--json", "7"]);

    match cli.command {
        Command::Show(args) => {
            assert_eq!(args.id, 7);
            assert!(args.json);
        }
        _ => panic!("expected show command"),
    }
}
