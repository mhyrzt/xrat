use super::*;

pub(super) async fn handle_runtime_disconnect(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<RuntimeDisconnectResult>,
) {
    let active_session_id = context
        .db
        .get_running_runtime_session()
        .await
        .ok()
        .flatten()
        .map(|session| session.id);
    match RuntimeService::new(context).disconnect().await {
        Ok(result) => {
            if result.stopped_session
                && let Some(session_id) = active_session_id
            {
                let _ = context
                    .db
                    .update_runtime_session_transition_metadata(
                        session_id,
                        Some("daemon"),
                        Some(&state.instance_id),
                        Some("manual_disconnect"),
                        Some("daemon runtime disconnect request succeeded"),
                        Some("daemon"),
                    )
                    .await;
            }
            if result.stopped_session {
                crate::app::events::record(
                    &context.db,
                    crate::app::events::LEVEL_INFO,
                    crate::app::events::SOURCE_RUNTIME,
                    "disconnect",
                    "Disconnected managed runtime",
                    None,
                    active_session_id,
                    None,
                )
                .await;
            }
            let _ = respond_to.send(RuntimeDisconnectResult::Ok(RuntimeDisconnectPayload {
                stopped_session: result.stopped_session,
            }));
        }
        Err(err) => {
            let _ = respond_to.send(RuntimeDisconnectResult::Err {
                message: err.to_string(),
            });
        }
    }
}
