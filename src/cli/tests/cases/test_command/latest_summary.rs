use clap::Parser;

use crate::cli::{Cli, Command};

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
