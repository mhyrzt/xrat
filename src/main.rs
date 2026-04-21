use xrat::app::{commands, runtime::AppContext};
use xrat::cli;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::parse();
    let context = AppContext::build(&args).await?;
    commands::run(&context, &args.command).await?;

    Ok(())
}
