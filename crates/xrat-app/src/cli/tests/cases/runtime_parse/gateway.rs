use crate::cli::{Cli, Command, GatewayAction};
use clap::Parser;

#[test]
fn parses_gateway_commands_and_repeated_subnets() {
    for action in ["doctor", "status", "remove"] {
        assert!(Cli::try_parse_from(["xrat", "gateway", action, "--json"]).is_ok());
    }
    let cli = Cli::try_parse_from([
        "xrat",
        "--config",
        "settings.toml",
        "gateway",
        "install",
        "--interface",
        "eth0",
        "--lan-subnet",
        "192.168.1.0/24",
        "--local-subnet",
        "192.168.2.0/24",
        "--local-subnet",
        "10.0.0.0/8",
        "--dry-run",
        "--json",
    ])
    .unwrap();
    let Command::Gateway(args) = cli.command else {
        panic!("gateway expected")
    };
    let GatewayAction::Install(args) = args.action else {
        panic!("install expected")
    };
    assert_eq!(args.local_subnets.len(), 2);
    assert!(args.dry_run && args.json);
}
