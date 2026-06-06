use clap::Parser;

use crate::cli::{Cli, Command, ListFormat, ListTarget};

#[test]
fn parses_import_subcommand_with_global_flags() {
    let cli = Cli::parse_from([
        "xrat",
        "--database",
        "/tmp/db.sqlite",
        "--config",
        "/tmp/config.toml",
        "--xray",
        "/opt/xray/xray",
        "--v2ray",
        "/opt/v2ray/v2ray",
        "--sing-box",
        "/opt/sing-box/sing-box",
        "import",
        "https://example.com/sub.txt",
    ]);

    assert_eq!(
        cli.database.as_deref(),
        Some(std::path::Path::new("/tmp/db.sqlite"))
    );
    assert_eq!(
        cli.config.as_deref(),
        Some(std::path::Path::new("/tmp/config.toml"))
    );
    assert_eq!(
        cli.xray.as_deref(),
        Some(std::path::Path::new("/opt/xray/xray"))
    );
    assert_eq!(
        cli.v2ray.as_deref(),
        Some(std::path::Path::new("/opt/v2ray/v2ray"))
    );
    assert_eq!(
        cli.sing_box.as_deref(),
        Some(std::path::Path::new("/opt/sing-box/sing-box"))
    );

    match cli.command {
        Command::Import(args) => assert_eq!(args.input, "https://example.com/sub.txt"),
        Command::Add(_) => panic!("expected import command"),
        Command::List(_) => panic!("expected import command"),
        Command::Test(_) => panic!("expected import command"),
        Command::Connect(_)
        | Command::Disconnect(_)
        | Command::Status(_)
        | Command::Daemon(_)
        | Command::Db(_)
        | Command::Rotate(_)
        | Command::Proxy(_)
        | Command::Serve(_)
        | Command::Tui(_)
        | Command::Parse(_)
        | Command::Validate(_)
        | Command::Upgrade(_)
        | Command::Version(_)
        | Command::Scan(_)
        | Command::Show(_)
        | Command::Enable(_)
        | Command::Disable(_)
        | Command::Delete(_)
        | Command::Restore(_)
        | Command::Purge(_)
        | Command::Logs(_)
        | Command::GeoIp(_)
        | Command::Init(_)
        | Command::Manpage(_)
        | Command::Completions(_) => {
            panic!("expected import command")
        }
    }
}

#[test]
fn import_without_input_reports_missing_argument() {
    let error = Cli::try_parse_from(["xrat", "import"]).expect_err("input should be required");
    let rendered = error.to_string();

    assert!(rendered.contains("required arguments were not provided"));
    assert!(rendered.contains("<INPUT>"));
    assert!(rendered.contains("Usage: xrat import <INPUT>"));
}

#[test]
fn parses_add_subcommand() {
    let cli = Cli::parse_from(["xrat", "add", "vless://example"]);

    match cli.command {
        Command::Add(args) => assert_eq!(args.input, "vless://example"),
        Command::Import(_) => panic!("expected add command"),
        Command::List(_) => panic!("expected add command"),
        Command::Test(_) => panic!("expected add command"),
        Command::Connect(_)
        | Command::Disconnect(_)
        | Command::Status(_)
        | Command::Daemon(_)
        | Command::Db(_)
        | Command::Rotate(_)
        | Command::Proxy(_)
        | Command::Serve(_)
        | Command::Tui(_)
        | Command::Parse(_)
        | Command::Validate(_)
        | Command::Upgrade(_)
        | Command::Version(_)
        | Command::Scan(_)
        | Command::Show(_)
        | Command::Enable(_)
        | Command::Disable(_)
        | Command::Delete(_)
        | Command::Restore(_)
        | Command::Purge(_)
        | Command::Logs(_)
        | Command::GeoIp(_)
        | Command::Init(_)
        | Command::Manpage(_)
        | Command::Completions(_) => {
            panic!("expected add command")
        }
    }
}

#[test]
fn parses_validate_subcommand() {
    let cli = Cli::parse_from(["xrat", "validate", "/tmp/config.toml"]);

    match cli.command {
        Command::Validate(args) => {
            assert_eq!(args.path, std::path::Path::new("/tmp/config.toml"));
        }
        _ => panic!("expected validate command"),
    }
}

#[test]
fn parses_list_subscriptions_alias() {
    let cli = Cli::parse_from(["xrat", "list", "subs"]);

    match cli.command {
        Command::List(args) => match args.target {
            ListTarget::Subscriptions(args) => assert!(matches!(args.format, ListFormat::Table)),
            ListTarget::Configs(_) => panic!("expected subscriptions target"),
        },
        Command::Import(_)
        | Command::Add(_)
        | Command::Test(_)
        | Command::Connect(_)
        | Command::Disconnect(_)
        | Command::Status(_)
        | Command::Daemon(_)
        | Command::Db(_)
        | Command::Rotate(_)
        | Command::Proxy(_)
        | Command::Serve(_)
        | Command::Tui(_)
        | Command::Parse(_)
        | Command::Validate(_)
        | Command::Upgrade(_)
        | Command::Version(_)
        | Command::Scan(_)
        | Command::Show(_)
        | Command::Enable(_)
        | Command::Disable(_)
        | Command::Delete(_)
        | Command::Restore(_)
        | Command::Purge(_)
        | Command::Logs(_)
        | Command::GeoIp(_)
        | Command::Init(_)
        | Command::Manpage(_)
        | Command::Completions(_) => {
            panic!("expected list command")
        }
    }
}

#[test]
fn list_without_target_reports_missing_list_subcommand() {
    let error = Cli::try_parse_from(["xrat", "list"]).expect_err("list target should be required");
    let rendered = error.to_string();

    assert!(rendered.contains("Usage: xrat list"));
    assert!(!rendered.contains("tui"));
}

#[test]
fn parses_list_config_filters() {
    let cli = Cli::parse_from([
        "xrat",
        "list",
        "configs",
        "--enabled-only",
        "--subscription",
        "7",
    ]);

    match cli.command {
        Command::List(args) => match args.target {
            ListTarget::Configs(filters) => {
                assert!(filters.enabled_only);
                assert_eq!(filters.subscription.as_deref(), Some("7"));
                assert!(matches!(filters.format, ListFormat::Table));
            }
            ListTarget::Subscriptions(_) => panic!("expected configs target"),
        },
        Command::Import(_)
        | Command::Add(_)
        | Command::Test(_)
        | Command::Connect(_)
        | Command::Disconnect(_)
        | Command::Status(_)
        | Command::Daemon(_)
        | Command::Db(_)
        | Command::Rotate(_)
        | Command::Proxy(_)
        | Command::Serve(_)
        | Command::Tui(_)
        | Command::Parse(_)
        | Command::Validate(_)
        | Command::Upgrade(_)
        | Command::Version(_)
        | Command::Scan(_)
        | Command::Show(_)
        | Command::Enable(_)
        | Command::Disable(_)
        | Command::Delete(_)
        | Command::Restore(_)
        | Command::Purge(_)
        | Command::Logs(_)
        | Command::GeoIp(_)
        | Command::Init(_)
        | Command::Manpage(_)
        | Command::Completions(_) => {
            panic!("expected list command")
        }
    }
}

#[test]
fn parses_list_output_formats() {
    let configs = Cli::parse_from(["xrat", "list", "configs", "--format", "json"]);
    match configs.command {
        Command::List(args) => match args.target {
            ListTarget::Configs(filters) => assert!(matches!(filters.format, ListFormat::Json)),
            ListTarget::Subscriptions(_) => panic!("expected configs target"),
        },
        _ => panic!("expected list command"),
    }

    let subscriptions = Cli::parse_from(["xrat", "list", "subscriptions", "--format", "tsv"]);
    match subscriptions.command {
        Command::List(args) => match args.target {
            ListTarget::Subscriptions(filters) => {
                assert!(matches!(filters.format, ListFormat::Tsv))
            }
            ListTarget::Configs(_) => panic!("expected subscriptions target"),
        },
        _ => panic!("expected list command"),
    }
}
