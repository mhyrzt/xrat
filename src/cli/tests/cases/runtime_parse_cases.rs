use clap::{CommandFactory, Parser};

use crate::cli::{Cli, Command, DaemonAction};

#[test]
fn parses_ping_loop_flags() {
    let cli = Cli::parse_from([
        "xrat",
        "test",
        "5",
        "--ping",
        "--ping-interval",
        "1500",
        "--upload-url",
        "https://example.com/upload",
    ]);
    match cli.command {
        Command::Test(args) => {
            assert_eq!(args.id, Some(5));
            assert!(args.ping);
            assert_eq!(args.ping_interval_ms, 1500);
            assert_eq!(
                args.upload_url.as_deref(),
                Some("https://example.com/upload")
            );
        }
        _ => panic!("expected test command"),
    }
}

#[test]
fn parses_scan_command() {
    let cli = Cli::parse_from([
        "xrat",
        "scan",
        "--ips",
        "1.1.1.1,8.8.8.8",
        "--port",
        "443",
        "--timeout",
        "5000",
    ]);
    match cli.command {
        Command::Scan(args) => {
            assert_eq!(args.ips.len(), 2);
            assert_eq!(args.port, 443);
            assert_eq!(args.timeout_ms, 5000);
        }
        _ => panic!("expected scan command"),
    }
}

#[test]
fn parses_runtime_commands() {
    let connect = Cli::parse_from(["xrat", "connect", "42", "--json"]);
    match connect.command {
        Command::Connect(args) => {
            assert_eq!(args.id, 42);
            assert!(args.json);
        }
        _ => panic!("expected connect command"),
    }

    let disconnect = Cli::parse_from(["xrat", "disconnect", "--json"]);
    match disconnect.command {
        Command::Disconnect(args) => assert!(args.json),
        _ => panic!("expected disconnect command"),
    }

    let status = Cli::parse_from(["xrat", "status", "--json"]);
    match status.command {
        Command::Status(args) => assert!(args.json),
        _ => panic!("expected status command"),
    }
}

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
            assert!(matches!(args.engine, crate::cli::ParseEngine::SingBox));
            assert!(args.input.is_none());
            assert!(args.file.is_none());
        }
        _ => panic!("expected parse command"),
    }
}

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
fn rejects_invalid_parse_engine_value() {
    let error = Cli::try_parse_from(["xrat", "parse", "--stdin", "--engine", "bad-engine"])
        .expect_err("invalid engine must fail");
    let rendered = error.to_string();
    assert!(rendered.contains("invalid value"));
    assert!(rendered.contains("sing-box"));
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
