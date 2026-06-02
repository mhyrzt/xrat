use clap::Parser;
use clap_complete::Shell;

use crate::cli::{Cli, Command};

#[test]
fn parses_completions_bash() {
    let cli = Cli::parse_from(["xrat", "completions", "bash"]);
    match cli.command {
        Command::Completions(args) => assert_eq!(args.shell, Shell::Bash),
        _ => panic!("expected completions command"),
    }
}

#[test]
fn parses_completions_zsh() {
    let cli = Cli::parse_from(["xrat", "completions", "zsh"]);
    match cli.command {
        Command::Completions(args) => assert_eq!(args.shell, Shell::Zsh),
        _ => panic!("expected completions command"),
    }
}

#[test]
fn parses_completions_fish() {
    let cli = Cli::parse_from(["xrat", "completions", "fish"]);
    match cli.command {
        Command::Completions(args) => assert_eq!(args.shell, Shell::Fish),
        _ => panic!("expected completions command"),
    }
}
