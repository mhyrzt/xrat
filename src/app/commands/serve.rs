use crate::app::context::AppContext;
use crate::cli::ServeArgs;

pub async fn run(context: &AppContext, args: &ServeArgs) -> crate::app::Result<()> {
    let mut settings = context.app_config.server.clone();
    if let Some(host) = &args.host {
        settings.host = host.clone();
    }
    if let Some(port) = args.port {
        settings.port = port;
    }

    crate::server::serve(context.db.clone(), &settings).await
}
