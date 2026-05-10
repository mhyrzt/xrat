use crate::app::daemon::server;
use crate::app::runtime::AppContext;
use crate::app::runtime_service::RuntimeService;
use crate::cli::StatusArgs;

mod display;
mod json;

pub async fn run(context: &AppContext, args: &StatusArgs) -> crate::app::Result<()> {
    let socket_path = server::default_socket_path(&context.runtime_paths.runtime_dir);
    match server::runtime_status_daemon(&socket_path).await {
        Ok(response) => return display::print_daemon_status(response, args.json),
        Err(err) if server::daemon_unreachable(&err) => {
            tracing::info!("daemon not reachable; using direct runtime status path");
        }
        Err(err) => return Err(err),
    }

    let snapshot = RuntimeService::new(context).status().await?;
    if args.json {
        json::print_json_status(&snapshot)
    } else {
        display::print_direct_status(snapshot)
    }
}
