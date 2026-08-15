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
    if let cli::Command::Validate(validate_args) = &args.command {
        return xrat::app::commands::validate::run(validate_args);
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
    async fn upgrade_bypasses_config_and_database_initialization() {
        let root = tempfile::tempdir().expect("temporary directory should exist");
        let config_path = root.path().join("invalid.toml");
        std::fs::write(&config_path, "this is not valid TOML = [")
            .expect("invalid config fixture should write");
        let args = xrat::cli::Cli::try_parse_from([
            "xrat",
            "--config",
            config_path.to_str().expect("config path should be UTF-8"),
            "upgrade",
            "--version",
            concat!("v", env!("CARGO_PKG_VERSION")),
        ])
        .expect("upgrade arguments should parse");

        super::run(&args)
            .await
            .expect("current-version upgrade should not initialize the database");
    }
}
