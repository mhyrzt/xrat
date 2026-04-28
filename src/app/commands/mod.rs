mod add;
mod import;
mod list;
mod test;

use crate::app::runtime::AppContext;
use crate::cli::Command;

pub async fn run(context: &AppContext, command: &Command) -> crate::app::Result<()> {
    match command {
        Command::Import(args) => import::run(context, &args.input).await,
        Command::Add(args) => add::run(context, &args.input).await,
        Command::List(args) => list::run(context, args).await,
        Command::Test(args) => test::run(args, context).await,
    }
}
