mod add;
mod completions;
mod connect;
mod daemon;
mod daemon_install;
mod db;
mod disconnect;
mod geoip;
mod import;
mod init;
mod lifecycle;
mod list;
mod logs;
mod manpage;
mod output;
mod parse;
mod progress;
mod proxy;
mod purge;
mod resolve;
mod rotate;
mod runtime_output;
mod scan;
mod serve;
mod setup;
mod status;
pub(crate) mod test;
mod tui;
pub(crate) mod update;
mod upgrade;
pub mod validate;
mod version;

use crate::app::context::AppContext;
use crate::cli::Command;

pub async fn run(context: &AppContext, command: &Command) -> crate::app::Result<()> {
    match command {
        Command::Init(args) => init::run(context, args),
        Command::Setup(args) => setup::run(context, args).await,
        Command::Import(args) => import::run(context, &args.input).await,
        Command::Add(args) => add::run(context, &args.input).await,
        Command::List(args) => list::run(context, args).await,
        Command::Show(args) => lifecycle::show(context, args).await,
        Command::Enable(args) => lifecycle::enable(context, args).await,
        Command::Disable(args) => lifecycle::disable(context, args).await,
        Command::Delete(args) => lifecycle::delete(context, args).await,
        Command::Restore(args) => lifecycle::restore(context, args).await,
        Command::Purge(args) => purge::run(context, args).await,
        Command::Test(args) => test::run(args, context).await,
        Command::Scan(args) => scan::run(context, args).await,
        Command::Connect(args) => connect::run(context, args).await,
        Command::Disconnect(args) => disconnect::run(context, args).await,
        Command::Status(args) => status::run(context, args).await,
        Command::Logs(args) => logs::run(context, args).await,
        Command::Daemon(args) => daemon::run(context, args).await,
        Command::Db(args) => db::run(context, args).await,
        Command::Rotate(args) => rotate::run(context, args).await,
        Command::Proxy(args) => proxy::run(context, args).await,
        Command::Serve(args) => serve::run(context, args).await,
        Command::Tui(args) => tui::run(context, args).await,
        Command::Update(args) => update::run(context, args).await,
        Command::Parse(args) => parse::run(args).await,
        Command::Validate(args) => validate::run(args),
        Command::Upgrade(args) => upgrade::run(context, args).await,
        Command::Version(args) => version::run(context, args),
        Command::Mmdb(args) => geoip::run(context, args).await,
        Command::Manpage(args) => manpage::run(context, args),
        Command::Completions(args) => completions::run(context, args),
    }
}
