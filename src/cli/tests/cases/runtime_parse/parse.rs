use clap::Parser;

use crate::cli::{Cli, Command, ParseEngine};

#[test]
fn parses_parse_command() {
    let cli = Cli::parse_from([
        "xrat",
        "parse",
        "--json",
        "--engine",
        "auto",
        "vless://example",
    ]);

    match cli.command {
        Command::Parse(args) => {
            assert!(args.json);
            assert_eq!(args.input.as_deref(), Some("vless://example"));
            assert!(!args.stdin);
            assert!(args.file.is_none());
        }
        _ => panic!("expected parse command"),
    }
}

#[test]
fn parses_parse_command_with_file_input() {
    let cli = Cli::parse_from(["xrat", "parse", "--file", "/tmp/links.txt"]);
    match cli.command {
        Command::Parse(args) => {
            assert_eq!(
                args.file.as_deref(),
                Some(std::path::Path::new("/tmp/links.txt"))
            );
            assert!(!args.stdin);
            assert!(args.input.is_none());
        }
        _ => panic!("expected parse command"),
    }
}

#[test]
fn parses_parse_command_with_stdin_input() {
    let cli = Cli::parse_from(["xrat", "parse", "--stdin", "--engine", "sing-box"]);
    match cli.command {
        Command::Parse(args) => {
            assert!(args.stdin);
            assert!(matches!(args.engine, ParseEngine::SingBox));
            assert!(args.input.is_none());
            assert!(args.file.is_none());
        }
        _ => panic!("expected parse command"),
    }
}

#[test]
fn rejects_invalid_parse_engine_value() {
    let error = Cli::try_parse_from(["xrat", "parse", "--stdin", "--engine", "bad-engine"])
        .expect_err("invalid engine must fail");
    let rendered = error.to_string();
    assert!(rendered.contains("invalid value"));
    assert!(rendered.contains("sing-box"));
}
