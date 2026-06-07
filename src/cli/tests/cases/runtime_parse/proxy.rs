use clap::Parser;

use crate::cli::{
    Cli, Command, ProxyAction, ProxyDesktopAction, ProxyPacAction, ProxyShellAction, ProxyShellKind,
};

#[test]
fn parses_proxy_endpoints() {
    let cli = Cli::parse_from(["xrat", "proxy", "info", "--json"]);
    match cli.command {
        Command::Proxy(args) => match args.action {
            ProxyAction::Info(info) => assert!(info.json),
            _ => panic!("expected info subcommand"),
        },
        _ => panic!("expected proxy command"),
    }
}

#[test]
fn parses_proxy_info_aliases() {
    for alias in ["show", "endpoints"] {
        let cli = Cli::parse_from(["xrat", "proxy", alias]);
        match cli.command {
            Command::Proxy(args) => match args.action {
                ProxyAction::Info(info) => assert!(!info.json),
                _ => panic!("expected info subcommand"),
            },
            _ => panic!("expected proxy command"),
        }
    }
}

#[test]
fn parses_proxy_pac_subcommands() {
    let url = Cli::parse_from(["xrat", "proxy", "pac", "url"]);
    match url.command {
        Command::Proxy(args) => match args.action {
            ProxyAction::Pac(pac) => assert!(matches!(pac.action, ProxyPacAction::Url(_))),
            _ => panic!("expected pac subcommand"),
        },
        _ => panic!("expected proxy command"),
    }

    let print = Cli::parse_from(["xrat", "proxy", "pac", "print"]);
    match print.command {
        Command::Proxy(args) => match args.action {
            ProxyAction::Pac(pac) => assert!(matches!(pac.action, ProxyPacAction::Print(_))),
            _ => panic!("expected pac subcommand"),
        },
        _ => panic!("expected proxy command"),
    }
}

#[test]
fn parses_proxy_shell_with_override() {
    let cli = Cli::parse_from(["xrat", "proxy", "shell", "enable", "--shell", "fish"]);
    match cli.command {
        Command::Proxy(args) => match args.action {
            ProxyAction::Shell(shell) => match shell.action {
                ProxyShellAction::Enable(enable) => {
                    assert_eq!(enable.shell, Some(ProxyShellKind::Fish));
                }
                _ => panic!("expected shell enable"),
            },
            _ => panic!("expected shell subcommand"),
        },
        _ => panic!("expected proxy command"),
    }
}

#[test]
fn parses_proxy_shell_toggle_with_override() {
    let cli = Cli::parse_from(["xrat", "proxy", "shell", "toggle", "--shell", "zsh"]);
    match cli.command {
        Command::Proxy(args) => match args.action {
            ProxyAction::Shell(shell) => match shell.action {
                ProxyShellAction::Toggle(toggle) => {
                    assert_eq!(toggle.shell, Some(ProxyShellKind::Zsh));
                }
                _ => panic!("expected shell toggle"),
            },
            _ => panic!("expected shell subcommand"),
        },
        _ => panic!("expected proxy command"),
    }
}

#[test]
fn parses_proxy_desktop_enable_pac() {
    let cli = Cli::parse_from(["xrat", "proxy", "desktop", "enable", "--pac"]);
    match cli.command {
        Command::Proxy(args) => match args.action {
            ProxyAction::Desktop(desktop) => match desktop.action {
                ProxyDesktopAction::Enable(enable) => assert!(enable.pac),
                _ => panic!("expected desktop enable"),
            },
            _ => panic!("expected desktop subcommand"),
        },
        _ => panic!("expected proxy command"),
    }
}

#[test]
fn parses_proxy_desktop_toggle_pac() {
    let cli = Cli::parse_from(["xrat", "proxy", "desktop", "toggle", "--pac"]);
    match cli.command {
        Command::Proxy(args) => match args.action {
            ProxyAction::Desktop(desktop) => match desktop.action {
                ProxyDesktopAction::Toggle(toggle) => assert!(toggle.pac),
                _ => panic!("expected desktop toggle"),
            },
            _ => panic!("expected desktop subcommand"),
        },
        _ => panic!("expected proxy command"),
    }
}

#[test]
fn old_proxy_rotation_commands_are_removed() {
    for removed in [
        ["xrat", "proxy", "start"],
        ["xrat", "proxy", "stop"],
        ["xrat", "proxy", "status"],
        ["xrat", "proxy", "toggle"],
    ] {
        assert!(
            Cli::try_parse_from(removed).is_err(),
            "{removed:?} should no longer parse"
        );
    }
}
