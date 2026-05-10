use crate::app::daemon::server;
use crate::app::runtime::AppContext;
use crate::app::runtime_service::RuntimeService;
use crate::cli::DisconnectArgs;

pub async fn run(context: &AppContext, args: &DisconnectArgs) -> crate::app::Result<()> {
    let socket_path = server::default_socket_path(&context.runtime_paths.runtime_dir);
    let result = match server::runtime_disconnect_daemon(&socket_path).await {
        Ok(response) => {
            if !response.ok {
                return Err(crate::app::AppError::InvalidArgument(response.message));
            }
            let payload = response.payload.ok_or_else(|| {
                crate::app::AppError::InvalidArgument(
                    "daemon disconnect response missing payload".to_string(),
                )
            })?;
            crate::app::runtime_service::DisconnectResult {
                stopped_session: payload.stopped_session,
            }
        }
        Err(err) if server::daemon_unreachable(&err) => {
            tracing::info!("daemon not reachable; using direct runtime disconnect path");
            RuntimeService::new(context).disconnect().await?
        }
        Err(err) => return Err(err),
    };

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "stopped_session": result.stopped_session,
                "message": if result.stopped_session {
                    "Disconnected active runtime session"
                } else {
                    "No active runtime session"
                },
            }))?
        );
        return Ok(());
    }

    if result.stopped_session {
        println!("Disconnected active runtime session");
    } else {
        println!("No active runtime session");
    }
    Ok(())
}
