use super::*;

pub async fn run(context: &AppContext, command: &Command) -> crate::app::Result<()> {
    match command {
        Command::Init(args) => init::run(context, args),
        Command::Setup(args) => setup::run(context, args).await,
        Command::Install(args) => setup::install(context, args).await,
        Command::Import(args) => import::run(context, args).await,
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
        Command::Webui(args) => serve::webui(context, args).await,
        Command::HashPassword(_) => serve::password_hash(),
        Command::Tui(args) => tui::run(context, args).await,
        Command::Tun(args) => tun::run(context, args).await,
        Command::Gateway(args) => {
            gateway::run_with_config(
                &context.app_config,
                args,
                &context.runtime_paths.runtime_dir,
            )
            .await
        }
        Command::Update(args) => update::run(context, args).await,
        Command::Parse(args) => parse::run(args).await,
        Command::Validate(args) => validate::run(args),
        Command::Upgrade(args) => upgrade::run(&context.runtime_paths.config_path, args).await,
        Command::Version(args) => version::run(context, args),
        Command::Mmdb(args) => geoip::run(context, args).await,
        Command::Manpage(args) => manpage::run(context, args),
        Command::Completions(args) => completions::run(context, args),
    }
}
