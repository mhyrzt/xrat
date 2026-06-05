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
