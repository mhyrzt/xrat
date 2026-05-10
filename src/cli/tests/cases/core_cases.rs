use clap::Parser;

use crate::cli::{Cli, Command, ListTarget};

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
        | Command::Parse(_)
        | Command::Scan(_) => {
            panic!("expected import command")
        }
    }
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
        | Command::Parse(_)
        | Command::Scan(_) => {
            panic!("expected add command")
        }
    }
}

#[test]
fn parses_list_subscriptions_alias() {
    let cli = Cli::parse_from(["xrat", "list", "subs"]);

    match cli.command {
        Command::List(args) => match args.target {
            ListTarget::Subscriptions(_) => {}
            ListTarget::Configs(_) => panic!("expected subscriptions target"),
        },
        Command::Import(_)
        | Command::Add(_)
        | Command::Test(_)
        | Command::Connect(_)
        | Command::Disconnect(_)
        | Command::Status(_)
        | Command::Daemon(_)
        | Command::Parse(_)
        | Command::Scan(_) => {
            panic!("expected list command")
        }
    }
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
                assert_eq!(filters.subscription, Some(7));
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
        | Command::Parse(_)
        | Command::Scan(_) => {
            panic!("expected list command")
        }
    }
}
