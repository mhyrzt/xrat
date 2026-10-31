use clap::Parser;

use crate::cli::{Cli, Command, TunAction};

#[test]
fn parses_tun_subcommands() {
    for (subcommand, enabled) in [("enable", true), ("disable", false)] {
        let cli = Cli::parse_from(["xrat", "tun", subcommand]);
        match cli.command {
            Command::Tun(args) => assert!(if enabled {
                matches!(args.action, TunAction::Enable(_))
            } else {
                matches!(args.action, TunAction::Disable(_))
            }),
            _ => panic!("expected tun command"),
        }
    }
    let status = Cli::parse_from(["xrat", "tun", "status"]);
    match status.command {
        Command::Tun(args) => assert!(matches!(args.action, TunAction::Status(_))),
        _ => panic!("expected tun command"),
    }

    let status_json = Cli::parse_from(["xrat", "tun", "status", "--json"]);
    match status_json.command {
        Command::Tun(args) => match args.action {
            TunAction::Status(status) => assert!(status.json),
            _ => panic!("expected tun status action"),
        },
        _ => panic!("expected tun command"),
    }

    let setup = Cli::parse_from(["xrat", "tun", "setup", "--dry-run"]);
    match setup.command {
        Command::Tun(args) => match args.action {
            TunAction::Setup(setup) => {
                assert!(setup.dry_run);
            }
            _ => panic!("expected tun setup action"),
        },
        _ => panic!("expected tun command"),
    }
}

#[test]
fn parses_tun_mode_json() {
    for command in ["enable", "disable"] {
        let cli = Cli::try_parse_from(["xrat", "tun", command, "--json"]).unwrap();
        let Command::Tun(args) = cli.command else {
            panic!("expected tun");
        };
        match args.action {
            TunAction::Enable(mode) | TunAction::Disable(mode) => assert!(mode.json),
            _ => panic!("expected a mode command"),
        }
    }
}

#[test]
fn parses_tun_split_subcommands() {
    use crate::cli::{
        ListFormat, TunSplitAction, TunSplitClearTarget, TunSplitListTarget,
        TunSplitListViewTarget, TunSplitModeArg,
    };

    let enable_whitelist =
        Cli::parse_from(["xrat", "tun", "enable", "--mode", "whitelist", "--json"]);
    match enable_whitelist.command {
        Command::Tun(args) => match args.action {
            TunAction::Enable(mode) => {
                assert_eq!(mode.mode, Some(TunSplitModeArg::Whitelist));
                assert!(mode.json);
            }
            _ => panic!("expected tun enable"),
        },
        _ => panic!("expected tun command"),
    }

    let set_mode = Cli::parse_from(["xrat", "tun", "mode", "blacklist"]);
    match set_mode.command {
        Command::Tun(args) => match args.action {
            TunAction::Mode(mode) => assert_eq!(mode.mode, TunSplitModeArg::Blacklist),
            _ => panic!("expected tun mode"),
        },
        _ => panic!("expected tun command"),
    }

    let add = Cli::parse_from([
        "xrat",
        "tun",
        "split",
        "add",
        "blacklist",
        "firefox",
        "/usr/bin/curl",
        "--json",
    ]);
    match add.command {
        Command::Tun(args) => match args.action {
            TunAction::Split(split) => match split.action {
                TunSplitAction::Add(modify) => {
                    assert_eq!(modify.list, TunSplitListTarget::Blacklist);
                    assert_eq!(modify.apps, vec!["firefox", "/usr/bin/curl"]);
                    assert!(modify.json);
                }
                _ => panic!("expected split add"),
            },
            _ => panic!("expected tun split"),
        },
        _ => panic!("expected tun command"),
    }

    let remove = Cli::parse_from(["xrat", "tun", "split", "remove", "whitelist", "firefox"]);
    match remove.command {
        Command::Tun(args) => match args.action {
            TunAction::Split(split) => match split.action {
                TunSplitAction::Remove(modify) => {
                    assert_eq!(modify.list, TunSplitListTarget::Whitelist);
                    assert_eq!(modify.apps, vec!["firefox"]);
                }
                _ => panic!("expected split remove"),
            },
            _ => panic!("expected tun split"),
        },
        _ => panic!("expected tun command"),
    }

    let clear = Cli::parse_from(["xrat", "tun", "split", "clear", "both"]);
    match clear.command {
        Command::Tun(args) => match args.action {
            TunAction::Split(split) => match split.action {
                TunSplitAction::Clear(clear) => {
                    assert_eq!(clear.list, TunSplitClearTarget::Both);
                }
                _ => panic!("expected split clear"),
            },
            _ => panic!("expected tun split"),
        },
        _ => panic!("expected tun command"),
    }

    let list = Cli::parse_from([
        "xrat",
        "tun",
        "split",
        "list",
        "whitelist",
        "--format",
        "json",
    ]);
    match list.command {
        Command::Tun(args) => match args.action {
            TunAction::Split(split) => match split.action {
                TunSplitAction::List(list) => {
                    assert_eq!(list.list, TunSplitListViewTarget::Whitelist);
                    assert_eq!(list.format, ListFormat::Json);
                }
                _ => panic!("expected split list"),
            },
            _ => panic!("expected tun split"),
        },
        _ => panic!("expected tun command"),
    }

    let apps = Cli::parse_from([
        "xrat",
        "tun",
        "split",
        "apps",
        "--running",
        "--search",
        "fire",
        "--format",
        "tsv",
    ]);
    match apps.command {
        Command::Tun(args) => match args.action {
            TunAction::Split(split) => match split.action {
                TunSplitAction::Apps(apps) => {
                    assert!(apps.running);
                    assert!(!apps.desktop);
                    assert_eq!(apps.search.as_deref(), Some("fire"));
                    assert_eq!(apps.format, ListFormat::Tsv);
                }
                _ => panic!("expected split apps"),
            },
            _ => panic!("expected tun split"),
        },
        _ => panic!("expected tun command"),
    }
}

#[test]
fn rejects_unsafe_sudoers_helper() {
    assert!(Cli::try_parse_from(["xrat", "tun", "setup", "--sudoers"]).is_err());
}

#[test]
fn parses_tun_killswitch_subcommands() {
    use crate::cli::TunKillswitchActionArg;

    let default_status = Cli::parse_from(["xrat", "tun", "killswitch"]);
    match default_status.command {
        Command::Tun(args) => match args.action {
            TunAction::Killswitch(ks) => {
                assert_eq!(ks.state, TunKillswitchActionArg::Status);
                assert!(!ks.json);
            }
            _ => panic!("expected tun killswitch"),
        },
        _ => panic!("expected tun command"),
    }

    for (arg_str, expected) in [
        ("status", TunKillswitchActionArg::Status),
        ("on", TunKillswitchActionArg::On),
        ("off", TunKillswitchActionArg::Off),
        ("reset", TunKillswitchActionArg::Reset),
    ] {
        let parsed = Cli::parse_from(["xrat", "tun", "killswitch", arg_str, "--json"]);
        match parsed.command {
            Command::Tun(args) => match args.action {
                TunAction::Killswitch(ks) => {
                    assert_eq!(ks.state, expected);
                    assert!(ks.json);
                }
                _ => panic!("expected tun killswitch"),
            },
            _ => panic!("expected tun command"),
        }
    }

    let enable_ks = Cli::parse_from(["xrat", "tun", "enable", "--kill-switch"]);
    match enable_ks.command {
        Command::Tun(args) => match args.action {
            TunAction::Enable(mode) => {
                assert!(mode.kill_switch);
            }
            _ => panic!("expected tun enable"),
        },
        _ => panic!("expected tun command"),
    }
}
