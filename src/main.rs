use xrat::app::{commands, context::AppContext};
use xrat::cli;

#[tokio::main]
async fn main() {
    let args = cli::parse();
    init_tracing(&args);

    if let Err(err) = run(&args).await {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

async fn run(args: &cli::Cli) -> xrat::app::Result<()> {
    if args.profile == cli::ExecutionProfile::Machine {
        if args.database.is_some()
            || args.config.is_some()
            || args.xray.is_some()
            || args.v2ray.is_some()
            || args.sing_box.is_some()
        {
            return Err(xrat::app::AppError::InvalidArgument(
                "machine profile does not accept database, config, or engine path overrides".into(),
            ));
        }
        #[cfg(windows)]
        return commands::machine::run(&args.command).await;
        #[cfg(not(windows))]
        return Err(xrat::app::AppError::UnsupportedPlatform(
            "machine profile requires Windows".into(),
        ));
    }
    #[cfg(windows)]
    if let cli::Command::Daemon(daemon) = &args.command {
        use xrat::app::commands::machine;
        match &daemon.action {
            cli::DaemonAction::Install(install) => return machine::install(install, false),
            cli::DaemonAction::Uninstall(uninstall) => return machine::uninstall(uninstall),
            _ => {}
        }
    }
    #[cfg(windows)]
    if matches!(
        &args.command,
        cli::Command::Daemon(cli::DaemonArgs {
            action: cli::DaemonAction::RunService
        })
    ) {
        return xrat::app::services::machine_service::run_service();
    }
    if let cli::Command::HashPassword(_) = &args.command {
        return xrat::app::commands::password_hash();
    }
    if let cli::Command::Validate(validate_args) = &args.command {
        return xrat::app::commands::validate::run(validate_args);
    }
    if let cli::Command::Gateway(gateway_args) = &args.command {
        return xrat::app::commands::gateway::run(args, gateway_args).await;
    }
    if let cli::Command::Upgrade(upgrade_args) = &args.command {
        let config_path = xrat::app::context::resolve_config_path(args)?;
        return xrat::app::commands::upgrade::run(&config_path, upgrade_args).await;
    }

    let context = AppContext::build(args).await?;
    commands::run(&context, &args.command).await?;

    Ok(())
}

fn init_tracing(args: &cli::Cli) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(args.default_log_filter()));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .compact()
        .init();
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    #[tokio::test]
    async fn gateway_dry_run_does_not_create_database_or_files() {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join("config.toml");
        let database = root.path().join("database.sqlite");
        std::fs::write(&config, "[gateway]\ninterface = 'eth0'\n").unwrap();
        let args = xrat::cli::Cli::try_parse_from([
            "xrat",
            "--config",
            config.to_str().unwrap(),
            "--database",
            database.to_str().unwrap(),
            "gateway",
            "install",
            "--dry-run",
        ])
        .unwrap();
        assert!(super::run(&args).await.is_err());
        assert!(!database.exists());
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
    }

    #[tokio::test]
    async fn gateway_doctor_and_rejected_install_preserve_existing_files() {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join("config.toml");
        let database = root.path().join("database.sqlite");
        let contents = "[gateway]\nlan_subnet = '192.168.1.0/24'\n";
        std::fs::write(&config, contents).unwrap();
        for arguments in [
            vec!["gateway", "doctor"],
            vec!["gateway", "install", "--lan-subnet", "192.168.1.1/24"],
        ] {
            let mut command = vec![
                "xrat",
                "--config",
                config.to_str().unwrap(),
                "--database",
                database.to_str().unwrap(),
            ];
            command.extend(arguments);
            let args = xrat::cli::Cli::try_parse_from(command).unwrap();
            let result = super::run(&args).await;
            if matches!(
                args.command,
                xrat::cli::Command::Gateway(xrat::cli::GatewayArgs {
                    action: xrat::cli::GatewayAction::Install(_)
                })
            ) {
                assert!(result.is_err());
            } else {
                assert!(result.is_ok());
            }
            assert_eq!(std::fs::read_to_string(&config).unwrap(), contents);
            assert!(!database.exists());
            assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
        }
    }

    #[tokio::test]
    async fn machine_profile_rejects_path_overrides_before_bootstrap() {
        let root = tempfile::tempdir().unwrap();
        let database = root.path().join("must-not-exist.sqlite");
        let args = xrat::cli::Cli::try_parse_from([
            "xrat",
            "--profile",
            "machine",
            "--database",
            database.to_str().unwrap(),
            "daemon",
            "status",
        ])
        .unwrap();
        let error = super::run(&args).await.unwrap_err();
        assert!(error.to_string().contains("does not accept"));
        assert!(!database.exists());
    }

    #[test]
    fn user_profile_remains_default_and_machine_selection_is_explicit() {
        let user = xrat::cli::Cli::try_parse_from(["xrat", "daemon", "status"]).unwrap();
        assert_eq!(user.profile, xrat::cli::ExecutionProfile::User);
        let machine =
            xrat::cli::Cli::try_parse_from(["xrat", "daemon", "status", "--profile", "machine"])
                .unwrap();
        assert_eq!(machine.profile, xrat::cli::ExecutionProfile::Machine);
    }

    #[tokio::test]
    async fn upgrade_bypasses_config_and_database_initialization() {
        let root = tempfile::tempdir().expect("temporary directory should exist");
        let config_path = root.path().join("invalid.toml");
        std::fs::write(&config_path, "this is not valid TOML = [")
            .expect("invalid config fixture should write");
        let database = root.path().join("must-not-exist.sqlite");
        let args = xrat::cli::Cli::try_parse_from([
            "xrat",
            "--config",
            config_path.to_str().expect("config path should be UTF-8"),
            "--database",
            database.to_str().unwrap(),
            "upgrade",
            "--source",
            "--path",
            root.path().to_str().unwrap(),
        ])
        .expect("upgrade arguments should parse");

        let error = super::run(&args).await.unwrap_err().to_string();
        #[cfg(windows)]
        let elevated = xrat_support::windows::is_current_process_elevated().unwrap();
        #[cfg(not(windows))]
        let elevated = false;
        assert!(
            error.contains(if elevated {
                "run user-profile upgrades without elevation"
            } else {
                "no Cargo.toml"
            }),
            "{error}"
        );
        assert!(!database.exists());
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
    }
}
