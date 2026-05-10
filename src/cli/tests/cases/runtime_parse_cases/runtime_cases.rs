use clap::Parser;

use crate::cli::{Cli, Command};

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
