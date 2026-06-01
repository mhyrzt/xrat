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
        | Command::Proxy(_)
        | Command::Serve(_)
        | Command::Tui(_)
        | Command::Parse(_)
        | Command::Scan(_)
        | Command::Show(_)
        | Command::Select(_)
        | Command::Enable(_)
        | Command::Disable(_)
        | Command::Delete(_)
        | Command::Restore(_) => {
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
        | Command::Proxy(_)
        | Command::Serve(_)
        | Command::Tui(_)
        | Command::Parse(_)
        | Command::Scan(_)
        | Command::Show(_)
        | Command::Select(_)
        | Command::Enable(_)
        | Command::Disable(_)
        | Command::Delete(_)
        | Command::Restore(_) => {
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
        | Command::Proxy(_)
        | Command::Serve(_)
        | Command::Tui(_)
        | Command::Parse(_)
        | Command::Scan(_)
        | Command::Show(_)
        | Command::Select(_)
        | Command::Enable(_)
        | Command::Disable(_)
        | Command::Delete(_)
        | Command::Restore(_) => {
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
        | Command::Proxy(_)
        | Command::Serve(_)
        | Command::Tui(_)
        | Command::Parse(_)
        | Command::Scan(_)
        | Command::Show(_)
        | Command::Select(_)
        | Command::Enable(_)
        | Command::Disable(_)
        | Command::Delete(_)
        | Command::Restore(_) => {
            panic!("expected list command")
        }
    }
}

#[test]
fn parses_serve_overrides() {
    let cli = Cli::parse_from(["xrat", "serve", "--host", "0.0.0.0", "--port", "9090"]);

    match cli.command {
        Command::Serve(args) => {
            assert_eq!(args.host.as_deref(), Some("0.0.0.0"));
            assert_eq!(args.port, Some(9090));
        }
        _ => panic!("expected serve command"),
    }
}

#[test]
fn parses_tui_subcommand() {
    let cli = Cli::parse_from(["xrat", "tui"]);

    match cli.command {
        Command::Tui(_) => {}
        _ => panic!("expected tui command"),
    }
}

#[test]
fn parses_select_subcommand() {
    let cli = Cli::parse_from(["xrat", "select", "42"]);

    match cli.command {
        Command::Select(args) => assert_eq!(args.id, 42),
        _ => panic!("expected select command"),
    }
}

#[test]
fn parses_enable_subcommand() {
    let cli = Cli::parse_from(["xrat", "enable", "7"]);

    match cli.command {
        Command::Enable(args) => assert_eq!(args.id, 7),
        _ => panic!("expected enable command"),
    }
}

#[test]
fn parses_disable_subcommand() {
    let cli = Cli::parse_from(["xrat", "disable", "7"]);

    match cli.command {
        Command::Disable(args) => assert_eq!(args.id, 7),
        _ => panic!("expected disable command"),
    }
}

#[test]
fn parses_delete_subcommand() {
    let cli = Cli::parse_from(["xrat", "delete", "7"]);

    match cli.command {
        Command::Delete(args) => {
            assert_eq!(args.id, 7);
            assert!(!args.hard);
        }
        _ => panic!("expected delete command"),
    }
}

#[test]
fn parses_delete_hard_subcommand() {
    let cli = Cli::parse_from(["xrat", "delete", "--hard", "7"]);

    match cli.command {
        Command::Delete(args) => {
            assert_eq!(args.id, 7);
            assert!(args.hard);
        }
        _ => panic!("expected delete command"),
    }
}

#[test]
fn parses_restore_subcommand() {
    let cli = Cli::parse_from(["xrat", "restore", "7"]);

    match cli.command {
        Command::Restore(args) => assert_eq!(args.id, 7),
        _ => panic!("expected restore command"),
    }
}

#[test]
fn parses_show_subcommand() {
    let cli = Cli::parse_from(["xrat", "show", "7"]);

    match cli.command {
        Command::Show(args) => {
            assert_eq!(args.id, 7);
            assert!(!args.json);
        }
        _ => panic!("expected show command"),
    }
}

#[test]
fn parses_show_json_subcommand() {
    let cli = Cli::parse_from(["xrat", "show", "--json", "7"]);

    match cli.command {
        Command::Show(args) => {
            assert_eq!(args.id, 7);
            assert!(args.json);
        }
        _ => panic!("expected show command"),
    }
}
