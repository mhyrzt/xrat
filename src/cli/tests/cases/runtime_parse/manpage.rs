use clap::Parser;

use crate::cli::{Cli, Command, ManpageArgs};

#[test]
fn parses_manpage_default_output() {
    let cli = Cli::parse_from(["xrat", "manpage"]);
    match cli.command {
        Command::Manpage(ManpageArgs { output }) => {
            assert_eq!(output, std::path::PathBuf::from("."));
        }
        _ => panic!("expected manpage command"),
    }
}

#[test]
fn parses_manpage_custom_output() {
    let cli = Cli::parse_from(["xrat", "manpage", "--output", "/tmp/man"]);
    match cli.command {
        Command::Manpage(ManpageArgs { output }) => {
            assert_eq!(output, std::path::PathBuf::from("/tmp/man"));
        }
        _ => panic!("expected manpage command"),
    }
}
