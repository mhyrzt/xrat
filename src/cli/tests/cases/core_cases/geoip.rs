use clap::Parser;

use crate::cli::{Cli, Command, GeoIpAction};

#[test]
fn parses_geoip_download_flags() {
    let cli = Cli::parse_from([
        "xrat",
        "geoip",
        "download",
        "--edition",
        "city",
        "--force",
        "--timeout",
        "30",
    ]);

    match cli.command {
        Command::GeoIp(args) => match args.action {
            GeoIpAction::Download(args) => {
                assert_eq!(args.editions, vec!["city"]);
                assert!(args.force);
                assert_eq!(args.timeout_secs, Some(30));
            }
            _ => panic!("expected geoip download command"),
        },
        _ => panic!("expected geoip command"),
    }
}

#[test]
fn parses_geoip_update_flags() {
    let cli = Cli::parse_from(["xrat", "geoip", "update", "--quiet"]);

    match cli.command {
        Command::GeoIp(args) => match args.action {
            GeoIpAction::Update(args) => assert!(args.quiet),
            _ => panic!("expected geoip update command"),
        },
        _ => panic!("expected geoip command"),
    }
}

#[test]
fn parses_geoip_path_subcommand() {
    let cli = Cli::parse_from(["xrat", "geoip", "path"]);

    match cli.command {
        Command::GeoIp(args) => match args.action {
            GeoIpAction::Path(args) => assert!(args.output.is_none()),
            _ => panic!("expected geoip path command"),
        },
        _ => panic!("expected geoip command"),
    }
}

#[test]
fn parses_geoip_status_flags() {
    let cli = Cli::parse_from([
        "xrat",
        "geoip",
        "status",
        "--strict",
        "--output",
        "./tmp/mmdb",
    ]);

    match cli.command {
        Command::GeoIp(args) => match args.action {
            GeoIpAction::Status(args) => {
                assert!(args.strict);
                assert_eq!(
                    args.output.as_deref(),
                    Some(std::path::Path::new("./tmp/mmdb"))
                );
            }
            _ => panic!("expected geoip status command"),
        },
        _ => panic!("expected geoip command"),
    }
}
