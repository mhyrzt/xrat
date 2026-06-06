use clap::Parser;

use crate::cli::{Cli, Command, ValidateFormat};

#[test]
fn parses_validate_command_with_default_format() {
    let cli = Cli::parse_from(["xrat", "validate", "config.toml"]);
    match cli.command {
        Command::Validate(args) => {
            assert_eq!(args.path.to_str(), Some("config.toml"));
            assert!(matches!(args.format, ValidateFormat::Human));
        }
        _ => panic!("expected validate command"),
    }
}

#[test]
fn parses_validate_command_with_json_format() {
    let cli = Cli::parse_from(["xrat", "validate", "--format", "json", "config.toml"]);
    match cli.command {
        Command::Validate(args) => {
            assert!(matches!(args.format, ValidateFormat::Json));
        }
        _ => panic!("expected validate command"),
    }
}

#[test]
fn requires_path_argument() {
    let error =
        Cli::try_parse_from(["xrat", "validate"]).expect_err("validate requires a path argument");
    assert!(error.to_string().contains("PATH") || error.to_string().contains("path"));
}

#[test]
fn rejects_invalid_format_value() {
    let error = Cli::try_parse_from(["xrat", "validate", "--format", "yaml", "config.toml"])
        .expect_err("invalid format must fail");
    assert!(error.to_string().contains("invalid value"));
}
