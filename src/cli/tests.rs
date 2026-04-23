#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::{Cli, Command, ListTarget};

    #[test]
    fn parses_import_subcommand_with_global_flags() {
        let cli = Cli::parse_from([
            "xrat",
            "--database",
            "/tmp/db.sqlite",
            "--config",
            "/tmp/Config.toml",
            "import",
            "https://example.com/sub.txt",
        ]);

        assert_eq!(
            cli.database.as_deref(),
            Some(std::path::Path::new("/tmp/db.sqlite"))
        );
        assert_eq!(
            cli.config.as_deref(),
            Some(std::path::Path::new("/tmp/Config.toml"))
        );

        match cli.command {
            Command::Import(args) => assert_eq!(args.input, "https://example.com/sub.txt"),
            Command::Add(_) => panic!("expected import command"),
            Command::List(_) => panic!("expected import command"),
        }
    }

    #[test]
    fn parses_add_subcommand() {
        let cli = Cli::parse_from(["xrat", "add", "vless://example"]);

        match cli.command {
            Command::Add(args) => assert_eq!(args.input, "vless://example"),
            Command::Import(_) => panic!("expected add command"),
            Command::List(_) => panic!("expected add command"),
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
            Command::Import(_) | Command::Add(_) => panic!("expected list command"),
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
            Command::Import(_) | Command::Add(_) => panic!("expected list command"),
        }
    }
}
