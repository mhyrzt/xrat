use clap::Parser;

use crate::cli::{Cli, Command, TestFormat, TestSortBy};

#[test]
fn parses_test_subcommand_flags() {
    let cli = Cli::parse_from([
        "xrat",
        "test",
        "42",
        "--skip-icmp",
        "--skip-real-delay",
        "--skip-download",
        "--test-url",
        "https://example.com/generate_204",
        "--download-url",
        "https://example.com/10mb.test",
        "--icmp-timeout",
        "3500",
        "--tcp-timeout",
        "4500",
        "--real-delay-timeout",
        "5500",
        "--download-timeout",
        "6500",
    ]);

    match cli.command {
        Command::Test(args) => {
            assert_eq!(args.id, Some(42));
            assert!(args.skip_icmp);
            assert!(args.skip_real_delay);
            assert!(args.skip_download);
            assert_eq!(
                args.test_url.as_deref(),
                Some("https://example.com/generate_204")
            );
            assert_eq!(
                args.download_url.as_deref(),
                Some("https://example.com/10mb.test")
            );
            assert_eq!(args.icmp_timeout_ms, Some(3500));
            assert_eq!(args.tcp_timeout_ms, Some(4500));
            assert_eq!(args.real_delay_timeout_ms, Some(5500));
            assert_eq!(args.download_timeout_ms, Some(6500));
        }
        Command::Import(_)
        | Command::Add(_)
        | Command::List(_)
        | Command::Connect(_)
        | Command::Disconnect(_)
        | Command::Status(_)
        | Command::Daemon(_)
        | Command::Parse(_)
        | Command::Scan(_) => {
            panic!("expected test command")
        }
    }
}

#[test]
fn parses_bulk_test_flags() {
    let cli = Cli::parse_from([
        "xrat",
        "test",
        "--enabled-only",
        "--subscription",
        "9",
        "--concurrency",
        "0",
        "--format",
        "csv",
        "--output",
        "/tmp/results.json",
        "--sort-by",
        "real-delay",
        "--no-progress",
    ]);

    match cli.command {
        Command::Test(args) => {
            assert_eq!(args.id, None);
            assert!(args.enabled_only);
            assert_eq!(args.subscription, Some(9));
            assert_eq!(args.concurrency, Some(0));
            assert!(matches!(args.format, TestFormat::Csv));
            assert_eq!(
                args.output.as_deref(),
                Some(std::path::Path::new("/tmp/results.json"))
            );
            assert!(matches!(args.sort_by, TestSortBy::RealDelay));
            assert!(args.no_progress);
            assert!(!args.latest_run_summary);
        }
        Command::Import(_)
        | Command::Add(_)
        | Command::List(_)
        | Command::Connect(_)
        | Command::Disconnect(_)
        | Command::Status(_)
        | Command::Daemon(_)
        | Command::Parse(_)
        | Command::Scan(_) => {
            panic!("expected test command")
        }
    }
}

#[test]
fn parses_csv_test_format() {
    let cli = Cli::parse_from(["xrat", "test", "--format", "csv"]);

    match cli.command {
        Command::Test(args) => assert!(matches!(args.format, TestFormat::Csv)),
        Command::Import(_)
        | Command::Add(_)
        | Command::List(_)
        | Command::Connect(_)
        | Command::Disconnect(_)
        | Command::Status(_)
        | Command::Daemon(_)
        | Command::Parse(_)
        | Command::Scan(_) => {
            panic!("expected test command")
        }
    }
}

#[test]
fn parses_latest_run_summary_flag() {
    let cli = Cli::parse_from(["xrat", "test", "--latest-run-summary"]);
    match cli.command {
        Command::Test(args) => {
            assert!(args.latest_run_summary);
            assert_eq!(args.id, None);
            assert_eq!(args.country, None);
            assert_eq!(args.asn, None);
        }
        _ => panic!("expected test command"),
    }
}

#[test]
fn parses_latest_run_summary_geo_filters() {
    let cli = Cli::parse_from([
        "xrat",
        "test",
        "--latest-run-summary",
        "--country",
        "US",
        "--asn",
        "cloudflare",
    ]);
    match cli.command {
        Command::Test(args) => {
            assert!(args.latest_run_summary);
            assert_eq!(args.country.as_deref(), Some("US"));
            assert_eq!(args.asn.as_deref(), Some("cloudflare"));
        }
        _ => panic!("expected test command"),
    }
}
