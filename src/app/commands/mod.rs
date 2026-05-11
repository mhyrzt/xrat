mod add;
mod connect;
mod daemon;
mod disconnect;
mod import;
mod list;
mod parse;
mod scan;
mod status;
mod test;

use crate::app::runtime::AppContext;
use crate::cli::Command;

pub async fn run(context: &AppContext, command: &Command) -> crate::app::Result<()> {
    match command {
        Command::Import(args) => import::run(context, &args.input).await,
        Command::Add(args) => add::run(context, &args.input).await,
        Command::List(args) => list::run(context, args).await,
        Command::Test(args) => test::run(args, context).await,
        Command::Scan(args) => scan::run(context, args).await,
        Command::Connect(args) => connect::run(context, args).await,
        Command::Disconnect(args) => disconnect::run(context, args).await,
        Command::Status(args) => status::run(context, args).await,
        Command::Daemon(args) => daemon::run(context, args).await,
        Command::Parse(args) => parse::run(args).await,
    }
}
