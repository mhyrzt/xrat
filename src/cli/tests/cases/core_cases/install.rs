use clap::Parser;
use semver::Version;

use crate::cli::{Cli, Command, InstallCore};

fn install(args: &[&str]) -> crate::cli::InstallArgs {
    let mut full = vec!["xrat", "install"];
    full.extend_from_slice(args);
    match Cli::parse_from(full).command {
        Command::Install(args) => args,
        _ => panic!("expected install command"),
    }
}

#[test]
fn parses_each_supported_core() {
    for (value, expected) in [
        ("xray", InstallCore::Xray),
        ("v2ray", InstallCore::V2Ray),
        ("sing-box", InstallCore::SingBox),
        ("singbox", InstallCore::SingBox),
    ] {
        assert_eq!(install(&[value]).core, expected);
    }
}

#[test]
fn defaults_to_latest_release() {
    let args = install(&["xray"]);
    assert_eq!(args.version, None);
    assert!(!args.prerelease);
}

#[test]
fn parses_pinned_version_with_or_without_v_prefix() {
    let expected = Some(Version::new(1, 13, 2));
    assert_eq!(
        install(&["sing-box", "--version", "1.13.2"]).version,
        expected
    );
    assert_eq!(
        install(&["sing-box", "--version", "v1.13.2"]).version,
        expected
    );
}

#[test]
fn parses_prerelease_and_rejects_it_with_a_pinned_version() {
    assert!(install(&["xray", "--prerelease"]).prerelease);
    assert!(
        Cli::try_parse_from([
            "xrat",
            "install",
            "xray",
            "--prerelease",
            "--version",
            "26.7.28",
        ])
        .is_err()
    );
}

#[test]
fn rejects_unknown_core_and_invalid_version() {
    assert!(Cli::try_parse_from(["xrat", "install", "unknown"]).is_err());
    assert!(Cli::try_parse_from(["xrat", "install", "xray", "--version", "current"]).is_err());
}
