use clap::{CommandFactory, Parser};

use crate::cli::{Cli, Command, DaemonAction, InitArgs};

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

    let restart = Cli::parse_from(["xrat", "daemon", "restart"]);
    match restart.command {
        Command::Daemon(args) => assert!(matches!(args.action, DaemonAction::Restart(_))),
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
fn parses_daemon_install_flags() {
    let cli = Cli::parse_from(["xrat", "daemon", "install"]);
    match cli.command {
        Command::Daemon(args) => match args.action {
            DaemonAction::Install(install_args) => {
                assert!(!install_args.start);
                assert!(!install_args.dry_run);
                assert!(!install_args.with_api);
            }
            _ => panic!("expected install action"),
        },
        _ => panic!("expected daemon command"),
    }

    let cli = Cli::parse_from(["xrat", "daemon", "install", "--start", "--with-api"]);
    match cli.command {
        Command::Daemon(args) => match args.action {
            DaemonAction::Install(install_args) => {
                assert!(install_args.start);
                assert!(install_args.with_api);
                assert!(!install_args.dry_run);
            }
            _ => panic!("expected install action"),
        },
        _ => panic!("expected daemon command"),
    }

    let cli = Cli::parse_from(["xrat", "daemon", "install", "--dry-run"]);
    match cli.command {
        Command::Daemon(args) => match args.action {
            DaemonAction::Install(install_args) => {
                assert!(install_args.dry_run);
            }
            _ => panic!("expected install action"),
        },
        _ => panic!("expected daemon command"),
    }
}

#[test]
fn parses_daemon_uninstall_flags() {
    let cli = Cli::parse_from(["xrat", "daemon", "uninstall"]);
    match cli.command {
        Command::Daemon(args) => match args.action {
            DaemonAction::Uninstall(uninstall_args) => {
                assert!(!uninstall_args.dry_run);
            }
            _ => panic!("expected uninstall action"),
        },
        _ => panic!("expected daemon command"),
    }

    let cli = Cli::parse_from(["xrat", "daemon", "uninstall", "--dry-run"]);
    match cli.command {
        Command::Daemon(args) => match args.action {
            DaemonAction::Uninstall(uninstall_args) => {
                assert!(uninstall_args.dry_run);
            }
            _ => panic!("expected uninstall action"),
        },
        _ => panic!("expected daemon command"),
    }
}

#[test]
fn parses_init_command() {
    let cli = Cli::parse_from(["xrat", "init"]);
    match cli.command {
        Command::Init(InitArgs { dry_run }) => assert!(!dry_run),
        _ => panic!("expected init command"),
    }

    let cli = Cli::parse_from(["xrat", "init", "--dry-run"]);
    match cli.command {
        Command::Init(InitArgs { dry_run }) => assert!(dry_run),
        _ => panic!("expected init command"),
    }
}

#[test]
fn parses_global_logging_flags() {
    let default_cli = Cli::parse_from(["xrat", "list", "configs"]);
    assert_eq!(default_cli.default_log_filter(), "error");

    let verbose_cli = Cli::parse_from(["xrat", "-vv", "list", "configs"]);
    assert_eq!(verbose_cli.verbose, 2);
    assert!(!verbose_cli.quiet);
    assert_eq!(verbose_cli.default_log_filter(), "debug");

    let quiet_cli = Cli::parse_from(["xrat", "--quiet", "-vvv", "list", "configs"]);
    assert_eq!(quiet_cli.verbose, 3);
    assert!(quiet_cli.quiet);
    assert_eq!(quiet_cli.default_log_filter(), "error");
}
