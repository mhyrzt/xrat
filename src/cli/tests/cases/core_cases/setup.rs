use clap::Parser;

use crate::cli::{Cli, Command, SetupFormat};

fn setup(args: &[&str]) -> crate::cli::SetupArgs {
    let mut full = vec!["xrat", "setup"];
    full.extend_from_slice(args);
    match Cli::parse_from(full).command {
        Command::Setup(args) => args,
        _ => panic!("expected setup command"),
    }
}

#[test]
fn parses_defaults() {
    let args = setup(&[]);
    assert!(!args.yes);
    assert!(!args.no_daemon);
    assert!(!args.no_desktop);
    assert!(!args.no_completions);
    assert!(!args.no_manpages);
    assert!(!args.linger);
    assert!(!args.check);
    assert_eq!(args.format, SetupFormat::Table);
}

#[test]
fn parses_all_mutating_flags() {
    let args = setup(&[
        "-y",
        "--no-daemon",
        "--no-desktop",
        "--no-completions",
        "--no-manpages",
    ]);
    assert!(args.yes);
    assert!(args.no_daemon);
    assert!(args.no_desktop);
    assert!(args.no_completions);
    assert!(args.no_manpages);
}

#[test]
fn parses_check_with_json_format() {
    let args = setup(&["--check", "--format", "json"]);
    assert!(args.check);
    assert_eq!(args.format, SetupFormat::Json);
}

#[test]
fn linger_implies_daemon_so_conflicts_with_no_daemon() {
    let result = Cli::try_parse_from(["xrat", "setup", "--linger", "--no-daemon"]);
    assert!(
        result.is_err(),
        "--linger with --no-daemon should be rejected"
    );
}

#[test]
fn check_rejects_mutating_flags() {
    for flag in [
        "-y",
        "--no-daemon",
        "--no-desktop",
        "--no-completions",
        "--no-manpages",
        "--linger",
    ] {
        let result = Cli::try_parse_from(["xrat", "setup", "--check", flag]);
        assert!(
            result.is_err(),
            "--check should reject mutating flag {flag}"
        );
    }
}
