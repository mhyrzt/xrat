use super::*;
use crate::app::daemon::ipc::RotationTrigger;

mod candidate;
mod stage;

impl<'a> RuntimeService<'a> {
    pub async fn replace(&self, request: ReplaceRequest) -> crate::app::Result<ReplaceResult> {
        let active = match self.active_session_state().await? {
            ActiveSessionState::Running(session) => session,
            ActiveSessionState::Stale(_) | ActiveSessionState::None => {
                let next_config_id = self.resolve_initial_rotation_candidate_id(&request).await?;
                let connected = self
                    .connect(ConnectRequest {
                        config_id: next_config_id,
                    })
                    .await?;
                self.context
                    .db
                    .update_runtime_session_transition_metadata(
                        connected.session_id,
                        None,
                        None,
                        Some("replace_commit_success"),
                        Some("runtime rotation started new session"),
                        Some("daemon"),
                    )
                    .await?;
                return Ok(ReplaceResult {
                    old_session_id: None,
                    new_config_id: connected.config.id,
                    new_session_id: connected.session_id,
                    new_pid: connected.pid,
                });
            }
        };
        let next_config_id = match self.resolve_replace_candidate_id(&active, &request).await {
            Ok(config_id) => config_id,
            Err(err) => {
                self.context
                    .db
                    .update_runtime_session_transition_metadata(
                        active.id,
                        None,
                        None,
                        Some("replace_rollback_keep_old"),
                        Some("replacement candidate rejected before handoff"),
                        Some("daemon"),
                    )
                    .await?;
                return Err(err);
            }
        };
        self.context
            .db
            .update_runtime_session_transition_metadata(
                active.id,
                None,
                None,
                Some("replace_started"),
                Some(&format!(
                    "trigger={:?}, candidate_id={}",
                    request.trigger, next_config_id
                )),
                Some("daemon"),
            )
            .await?;

        stop_session(self.context, &active).await?;
        self.context.db.clear_active_config().await?;

        let staged = self.stage_replacement_runtime(next_config_id).await;
        let (next_config_id, session_id, new_pid) = match staged {
            Ok(value) => value,
            Err(err) => {
                self.context.db.clear_active_config().await?;
                return Err(err);
            }
        };

        self.context.db.set_active_config(next_config_id).await?;
        self.context
            .db
            .update_runtime_session_transition_metadata(
                session_id,
                None,
                None,
                Some("replace_commit_success"),
                Some("runtime replace handoff completed"),
                Some("daemon"),
            )
            .await?;

        Ok(ReplaceResult {
            old_session_id: Some(active.id),
            new_config_id: next_config_id,
            new_session_id: session_id,
            new_pid,
        })
    }
}
