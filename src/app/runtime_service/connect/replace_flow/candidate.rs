use super::*;
use crate::app::commands::test::run_rotation_bulk_tests;
use crate::support::time::now_epoch_seconds;

impl<'a> RuntimeService<'a> {
    pub(super) async fn resolve_replace_candidate_id(
        &self,
        active: &RuntimeSessionRecord,
        request: &ReplaceRequest,
    ) -> crate::app::Result<i64> {
        if let Some(candidate_id) = request.candidate_id {
            let Some(config) = self.context.db.get_config_by_id(candidate_id).await? else {
                return Err(AppError::InvalidArgument(format!(
                    "config {} was not found",
                    candidate_id
                )));
            };
            if !config.is_enabled {
                return Err(AppError::InvalidArgument(format!(
                    "config {} is disabled and cannot be selected for replacement",
                    candidate_id
                )));
            }
            if !matches!(request.trigger, RotationTrigger::Manual)
                && self.config_is_on_cooldown(candidate_id).await?
            {
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
        let filter = ConfigListFilter {
            only_enabled: true,
            ..Default::default()
        };
        let configs = self.context.db.list_configs(&filter).await?;
        let mut eligible_ids: Vec<i64> = Vec::new();
        let mut passing: Vec<(i64, i64, Option<f64>)> = Vec::new();
        for config in configs.into_iter().filter(|cfg| cfg.id != active_config_id) {
            if !matches!(request.trigger, RotationTrigger::Manual)
                && self.config_is_on_cooldown(config.id).await?
            {
                continue;
            }
            eligible_ids.push(config.id);
        }

        if !matches!(request.trigger, RotationTrigger::Manual) {
            let _ = run_rotation_bulk_tests(self.context, &eligible_ids).await;
        }

        for config_id in &eligible_ids {
            let Some(test) = self
                .context
                .db
                .get_latest_connection_test(*config_id)
                .await?
            else {
                continue;
            };
            if test.real_delay_ok != Some(true) {
                continue;
            }
            let Some(real_delay_ms) = test.real_delay_ms else {
                continue;
            };
            passing.push((*config_id, real_delay_ms, test.download_mbps));
        }
        passing.sort_by(|left, right| {
            left.1
                .cmp(&right.1)
                .then_with(|| compare_optional_f64_desc(left.2, right.2))
                .then_with(|| left.0.cmp(&right.0))
        });
        if let Some((config_id, _, _)) = passing.first().copied() {
            return Ok(config_id);
        }
        if matches!(request.trigger, RotationTrigger::Manual)
            && let Some(config_id) = eligible_ids.into_iter().min()
        {
            return Ok(config_id);
        }

        Err(AppError::InvalidArgument(
            "no eligible replacement candidate".to_string(),
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

fn compare_optional_f64_desc(left: Option<f64>, right: Option<f64>) -> std::cmp::Ordering {
    match (left, right) {
        (Some(a), Some(b)) => b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}
