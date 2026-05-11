use super::super::*;
use super::stale::mark_session_stale;

pub(crate) async fn active_session_state(
    context: &AppContext,
) -> crate::app::Result<ActiveSessionState> {
    let Some(session) = context.db.get_running_runtime_session().await? else {
        return Ok(ActiveSessionState::None);
    };
    if runtime_session_is_alive(&session) {
        return Ok(ActiveSessionState::Running(session));
    }
    mark_session_stale(context, &session).await?;
    Ok(ActiveSessionState::Stale(session))
}

pub(crate) fn runtime_session_is_alive(session: &RuntimeSessionRecord) -> bool {
    session
        .process_id
        .map(xray_runtime::process_is_running)
        .unwrap_or(false)
}
