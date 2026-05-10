use super::*;

impl<'a> RuntimeService<'a> {
    pub async fn replace(&self, request: ReplaceRequest) -> crate::app::Result<ReplaceResult> {
        let active = match self.active_session_state().await? {
            ActiveSessionState::Running(session) => session,
            ActiveSessionState::Stale(_) | ActiveSessionState::None => {
                return Err(AppError::InvalidArgument(
                    "no running runtime session to replace".to_string(),
                ));
            }
        };
        let next_config_id = request
            .candidate_id
            .unwrap_or(active.config_id.ok_or_else(|| {
                AppError::InvalidArgument("active runtime session has no config id".to_string())
            })?);
        let result = self
            .connect(ConnectRequest {
                config_id: next_config_id,
            })
            .await?;
        Ok(ReplaceResult {
            old_session_id: active.id,
            new_config_id: result.config.id,
            new_session_id: result.session_id,
            new_pid: result.pid,
        })
    }
}
