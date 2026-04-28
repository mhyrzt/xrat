use xrat::app::{commands, runtime::AppContext};
use xrat::cli;

#[tokio::main]
async fn main() {
    init_tracing();

    if let Err(err) = run().await {
        tracing::error!(error = %err, "command failed");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::parse();
    let context = AppContext::build(&args).await?;
    commands::run(&context, &args.command).await?;

    Ok(())
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .compact()
        .init();
}
