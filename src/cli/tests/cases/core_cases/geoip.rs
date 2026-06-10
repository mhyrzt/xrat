use clap::Parser;

use crate::cli::{Cli, Command, GeoIpAction};

#[test]
fn parses_geoip_download_flags() {
    let cli = Cli::parse_from([
        "xrat",
        "mmdb",
        "download",
        "--edition",
        "city",
        "--force",
        "--timeout",
        "30",
    ]);

    match cli.command {
        Command::Mmdb(args) => match args.action {
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
    let cli = Cli::parse_from(["xrat", "mmdb", "update", "--quiet"]);

    match cli.command {
        Command::Mmdb(args) => match args.action {
            GeoIpAction::Update(args) => assert!(args.quiet),
            _ => panic!("expected geoip update command"),
        },
        _ => panic!("expected geoip command"),
    }
}

#[test]
fn parses_geoip_lookup_flags() {
    let cli = Cli::parse_from([
        "xrat",
        "mmdb",
        "lookup",
        "google.com",
        "--backend",
        "ipwhois",
        "--no-cache",
        "--json",
    ]);

    match cli.command {
        Command::Mmdb(args) => match args.action {
            GeoIpAction::Lookup(args) => {
                assert_eq!(args.ip, "google.com");
                assert_eq!(args.backend.as_deref(), Some("ipwhois"));
                assert!(args.no_cache);
                assert!(args.json);
            }
            _ => panic!("expected geoip lookup command"),
        },
        _ => panic!("expected geoip command"),
    }
}

#[test]
fn parses_geoip_backend_flags() {
    let cli = Cli::parse_from([
        "xrat",
        "mmdb",
        "backend",
        "--backend",
        "ip-api",
        "--no-cache",
    ]);

    match cli.command {
        Command::Mmdb(args) => match args.action {
            GeoIpAction::Backend(args) => {
                assert_eq!(args.backend.as_deref(), Some("ip-api"));
                assert!(args.no_cache);
            }
            _ => panic!("expected geoip backend command"),
        },
        _ => panic!("expected geoip command"),
    }
}

#[test]
fn parses_geoip_path_subcommand() {
    let cli = Cli::parse_from(["xrat", "mmdb", "path"]);

    match cli.command {
        Command::Mmdb(args) => match args.action {
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
        "mmdb",
        "status",
        "--strict",
        "--json",
        "--output",
        "./tmp/mmdb",
    ]);

    match cli.command {
        Command::Mmdb(args) => match args.action {
            GeoIpAction::Status(args) => {
                assert!(args.strict);
                assert!(args.json);
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

#[test]
fn rejects_old_geoip_namespace() {
    assert!(Cli::try_parse_from(["xrat", "geoip", "status"]).is_err());
}
