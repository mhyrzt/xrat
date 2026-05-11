use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

impl<'a> RuntimeService<'a> {
    pub(super) async fn resolve_replace_candidate_id(
        &self,
        active: &RuntimeSessionRecord,
        request: &ReplaceRequest,
    ) -> crate::app::Result<i64> {
        if let Some(candidate_id) = request.candidate_id {
            if self.config_is_on_cooldown(candidate_id).await? {
                return Err(AppError::InvalidArgument(format!(
                    "config {} is on cooldown and cannot be selected for replacement",
                    candidate_id
                )));
            }
            return Ok(candidate_id);
        }

        let active_config_id = active.config_id.ok_or_else(|| {
            AppError::InvalidArgument("active runtime session has no config id".to_string())
        })?;
        if matches!(request.trigger, RotationTrigger::Manual) {
            return Ok(active_config_id);
        }

        let filter = ConfigListFilter {
            only_enabled: true,
            ..Default::default()
        };
        let configs = self.context.db.list_configs(&filter).await?;
        for config in configs.into_iter().filter(|cfg| cfg.id != active_config_id) {
            if !self.config_is_on_cooldown(config.id).await? {
                return Ok(config.id);
            }
        }

        Err(AppError::InvalidArgument(
            "no eligible replacement candidate: all enabled alternatives are on cooldown"
                .to_string(),
        ))
    }

    async fn config_is_on_cooldown(&self, config_id: i64) -> crate::app::Result<bool> {
        let Some(session) = self
            .context
            .db
            .get_latest_runtime_session_for_config(config_id)
            .await?
        else {
            return Ok(false);
        };
        let Some(cooldown_until) = session.cooldown_until.as_deref() else {
            return Ok(false);
        };
        let Ok(cooldown_until) = cooldown_until.parse::<u64>() else {
            return Ok(false);
        };
        Ok(now_epoch_seconds() < cooldown_until)
    }
}

fn now_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
