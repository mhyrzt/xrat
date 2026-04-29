use crate::app::runtime::AppContext;
use crate::cli::DisconnectArgs;
use crate::db::RuntimeSessionStatus;
use crate::xray::runtime as xray_runtime;

pub async fn run(context: &AppContext, _args: &DisconnectArgs) -> crate::app::Result<()> {
    if disconnect_active(context).await? {
        println!("Disconnected active runtime session");
    } else {
        println!("No active runtime session");
    }
    Ok(())
}

pub async fn disconnect_active(context: &AppContext) -> crate::app::Result<bool> {
    let Some(session) = context.db.get_running_runtime_session().await? else {
        context.db.clear_active_config().await?;
        return Ok(false);
    };

    context
        .db
        .update_runtime_session_state(
            session.id,
            RuntimeSessionStatus::Stopping,
            None,
            None,
            None,
            None,
        )
        .await?;

    if let Some(pid) = session.process_id {
        if xray_runtime::process_is_running(pid) {
            let _ = xray_runtime::terminate_process(pid)?;
        } else {
            tracing::warn!(
                session_id = session.id,
                pid,
                "runtime process was already gone"
            );
        }
    }

    context
        .db
        .mark_runtime_session_stopped(session.id, None)
        .await?;
    context.db.clear_active_config().await?;
    Ok(true)
}
