mod add;
mod import;
mod list;

use crate::app::runtime::AppContext;
use crate::cli::Command;

pub async fn run(
    context: &AppContext,
    command: &Command,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Command::Import(args) => import::run(context, &args.input).await,
        Command::Add(args) => add::run(context, &args.input).await,
        Command::List(args) => list::run(context, args).await,
    }
}
