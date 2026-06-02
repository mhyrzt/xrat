mod add;
mod completions;
mod connect;
mod daemon;
mod daemon_install;
mod disconnect;
mod geoip;
mod import;
mod init;
mod lifecycle;
mod list;
mod manpage;
mod parse;
mod proxy;
mod scan;
mod serve;
mod status;
pub(crate) mod test;
mod tui;

use crate::app::context::AppContext;
use crate::cli::Command;

pub async fn run(context: &AppContext, command: &Command) -> crate::app::Result<()> {
    match command {
        Command::Init(args) => init::run(context, args),
        Command::Import(args) => import::run(context, &args.input).await,
        Command::Add(args) => add::run(context, &args.input).await,
        Command::List(args) => list::run(context, args).await,
        Command::Show(args) => lifecycle::show(context, args).await,
        Command::Select(args) => lifecycle::select(context, args).await,
        Command::Enable(args) => lifecycle::enable(context, args).await,
        Command::Disable(args) => lifecycle::disable(context, args).await,
        Command::Delete(args) => lifecycle::delete(context, args).await,
        Command::Restore(args) => lifecycle::restore(context, args).await,
        Command::Test(args) => test::run(args, context).await,
        Command::Scan(args) => scan::run(context, args).await,
        Command::Connect(args) => connect::run(context, args).await,
        Command::Disconnect(args) => disconnect::run(context, args).await,
        Command::Status(args) => status::run(context, args).await,
        Command::Daemon(args) => daemon::run(context, args).await,
        Command::Proxy(args) => proxy::run(context, args).await,
        Command::Serve(args) => serve::run(context, args).await,
        Command::Tui(args) => tui::run(context, args).await,
        Command::Parse(args) => parse::run(args).await,
        Command::GeoIp(args) => geoip::run(context, args).await,
        Command::Manpage(args) => manpage::run(context, args),
        Command::Completions(args) => completions::run(context, args),
    }
}
