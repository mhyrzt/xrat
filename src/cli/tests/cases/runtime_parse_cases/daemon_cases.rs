use clap::{CommandFactory, Parser};

use crate::cli::{Cli, Command, DaemonAction};

#[test]
fn parses_daemon_subcommands() {
    let start = Cli::parse_from(["xrat", "daemon", "start"]);
    match start.command {
        Command::Daemon(args) => assert!(matches!(args.action, DaemonAction::Start(_))),
        _ => panic!("expected daemon command"),
    }

    let status = Cli::parse_from(["xrat", "daemon", "status"]);
    match status.command {
        Command::Daemon(args) => assert!(matches!(args.action, DaemonAction::Status(_))),
        _ => panic!("expected daemon command"),
    }

    let stop = Cli::parse_from(["xrat", "daemon", "stop"]);
    match stop.command {
        Command::Daemon(args) => assert!(matches!(args.action, DaemonAction::Stop(_))),
        _ => panic!("expected daemon command"),
    }
}

#[test]
fn daemon_stop_help_describes_shutdown_contract() {
    let mut cmd = Cli::command();
    let daemon_cmd = cmd
        .find_subcommand_mut("daemon")
        .expect("daemon subcommand should exist");
    let stop_cmd = daemon_cmd
        .find_subcommand_mut("stop")
        .expect("daemon stop subcommand should exist");
    let about = stop_cmd
        .get_about()
        .map(|v| v.to_string())
        .unwrap_or_default();
    assert!(
        about.contains("Request daemon shutdown via local IPC."),
        "daemon stop help text should describe shutdown over IPC"
    );
}

#[test]
fn parses_global_logging_flags() {
    let verbose_cli = Cli::parse_from(["xrat", "-vv", "list", "configs"]);
    assert_eq!(verbose_cli.verbose, 2);
    assert!(!verbose_cli.quiet);
    assert_eq!(verbose_cli.default_log_filter(), "debug");

    let quiet_cli = Cli::parse_from(["xrat", "--quiet", "-vvv", "list", "configs"]);
    assert_eq!(quiet_cli.verbose, 3);
    assert!(quiet_cli.quiet);
    assert_eq!(quiet_cli.default_log_filter(), "error");
}
