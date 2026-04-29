use crate::app::runtime::AppContext;
use crate::app::runtime_service::RuntimeService;
use crate::cli::DisconnectArgs;

pub async fn run(context: &AppContext, args: &DisconnectArgs) -> crate::app::Result<()> {
    let result = RuntimeService::new(context).disconnect().await?;

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
