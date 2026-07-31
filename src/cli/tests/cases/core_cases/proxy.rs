use clap::{CommandFactory, Parser};

use crate::cli::{Cli, Command, ProxyAction, ProxyShellAction, ProxyShellKind, ProxyShellProtocol};

#[test]
fn parses_proxy_shell_enable_with_protocol_positional() {
    let cli = Cli::parse_from(["xrat", "proxy", "shell", "enable", "socks5h"]);

    match cli.command {
        Command::Proxy(args) => match args.action {
            ProxyAction::Shell(shell) => match shell.action {
                ProxyShellAction::Enable(args) => {
                    assert_eq!(args.protocol, Some(ProxyShellProtocol::Socks5h));
                    assert_eq!(args.shell, None);
                }
                _ => panic!("expected proxy shell enable command"),
            },
            _ => panic!("expected proxy shell command"),
        },
        _ => panic!("expected proxy command"),
    }
}

#[test]
fn parses_proxy_shell_enable_protocol_values() {
    for (raw, expected) in [
        ("http", ProxyShellProtocol::Http),
        ("socks5", ProxyShellProtocol::Socks5),
        ("socks5h", ProxyShellProtocol::Socks5h),
    ] {
        let cli = Cli::parse_from(["xrat", "proxy", "shell", "enable", raw]);
        match cli.command {
            Command::Proxy(args) => match args.action {
                ProxyAction::Shell(shell) => match shell.action {
                    ProxyShellAction::Enable(args) => {
                        assert_eq!(args.protocol, Some(expected));
                    }
                    _ => panic!("expected proxy shell enable command"),
                },
                _ => panic!("expected proxy shell command"),
            },
            _ => panic!("expected proxy command"),
        }
    }
}

#[test]
fn parses_proxy_shell_enable_without_protocol_defaults_to_none() {
    let cli = Cli::parse_from(["xrat", "proxy", "shell", "enable"]);

    match cli.command {
        Command::Proxy(args) => match args.action {
            ProxyAction::Shell(shell) => match shell.action {
                ProxyShellAction::Enable(args) => {
                    assert_eq!(args.protocol, None);
                    assert_eq!(args.shell, None);
                }
                _ => panic!("expected proxy shell enable command"),
            },
            _ => panic!("expected proxy shell command"),
        },
        _ => panic!("expected proxy command"),
    }
}

#[test]
fn parses_proxy_shell_enable_with_shell_and_protocol() {
    let cli = Cli::parse_from([
        "xrat", "proxy", "shell", "enable", "--shell", "fish", "http",
    ]);

    match cli.command {
        Command::Proxy(args) => match args.action {
            ProxyAction::Shell(shell) => match shell.action {
                ProxyShellAction::Enable(args) => {
                    assert_eq!(args.shell, Some(ProxyShellKind::Fish));
                    assert_eq!(args.protocol, Some(ProxyShellProtocol::Http));
                }
                _ => panic!("expected proxy shell enable command"),
            },
            _ => panic!("expected proxy shell command"),
        },
        _ => panic!("expected proxy command"),
    }
}

#[test]
fn rejects_unknown_proxy_shell_protocol() {
    let error = Cli::try_parse_from(["xrat", "proxy", "shell", "enable", "bogus"])
        .expect_err("unknown protocol should be rejected");
    let rendered = error.to_string();
    assert!(rendered.contains("invalid value"));
    assert!(rendered.contains("http"));
    assert!(rendered.contains("socks5"));
    assert!(rendered.contains("socks5h"));
}

#[test]
fn proxy_shell_help_shows_shell_specific_usage() {
    let mut cmd = Cli::command();
    let proxy_cmd = cmd
        .find_subcommand_mut("proxy")
        .expect("proxy subcommand should exist");
    let shell_cmd = proxy_cmd
        .find_subcommand_mut("shell")
        .expect("proxy shell subcommand should exist");

    for (name, expected) in [
        ("enable", "eval \"$(xrat proxy shell enable)\""),
        ("disable", "eval \"$(xrat proxy shell disable)\""),
        ("toggle", "eval \"$(xrat proxy shell toggle)\""),
    ] {
        let sub = shell_cmd
            .find_subcommand_mut(name)
            .expect("proxy shell subcommand should exist");
        let long_about = sub
            .get_long_about()
            .map(|v| v.to_string())
            .unwrap_or_default();
        assert!(
            long_about.contains(expected),
            "proxy shell {name} help should mention {expected}"
        );
        assert!(
            long_about.contains("fish"),
            "proxy shell {name} help should mention fish usage"
        );
    }
}
