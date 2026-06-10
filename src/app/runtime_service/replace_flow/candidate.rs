use super::*;
use crate::app::commands::test::run_rotation_bulk_tests;
use crate::app::{events, subscription_refresh};
use crate::support::time::now_epoch_seconds;

impl<'a> RuntimeService<'a> {
    pub(super) async fn resolve_initial_rotation_candidate_id(
        &self,
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

        self.refresh_subscriptions_before_automatic_rotation(request)
            .await;

        let filter = ConfigListFilter {
            only_enabled: true,
            ..Default::default()
        };
        let configs = self.context.db.list_configs(&filter).await?;
        let mut eligible_ids: Vec<i64> = Vec::new();
        let mut passing: Vec<(i64, i64, Option<f64>)> = Vec::new();
        for config in configs {
            if !matches!(request.trigger, RotationTrigger::Manual)
                && self.config_is_on_cooldown(config.id).await?
            {
                continue;
            }
            eligible_ids.push(config.id);
        }

        self.collect_passing_rotation_candidates(request, &eligible_ids, &mut passing)
            .await?;

        passing.sort_by(|left, right| {
            left.1
                .cmp(&right.1)
                .then_with(|| compare_optional_f64_desc(left.2, right.2))
                .then_with(|| left.0.cmp(&right.0))
        });
        if let Some((config_id, _, _)) = passing.first().copied() {
            return Ok(config_id);
        }

        Err(no_passing_candidate_error())
    }

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

        self.refresh_subscriptions_before_automatic_rotation(request)
            .await;

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

        self.collect_passing_rotation_candidates(request, &eligible_ids, &mut passing)
            .await?;
        passing.sort_by(|left, right| {
            left.1
                .cmp(&right.1)
                .then_with(|| compare_optional_f64_desc(left.2, right.2))
                .then_with(|| left.0.cmp(&right.0))
        });
        if let Some((config_id, _, _)) = passing.first().copied() {
            return Ok(config_id);
        }

        Err(no_passing_candidate_error())
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

    async fn refresh_subscriptions_before_automatic_rotation(&self, request: &ReplaceRequest) {
        if matches!(request.trigger, RotationTrigger::Manual)
            || !self
                .context
                .app_config
                .runtime
                .rotation
                .refresh_subscriptions
        {
            return;
        }

        let outcome = subscription_refresh::refresh_all(self.context).await;
        if outcome.attempted > 0 {
            events::record(
                &self.context.db,
                if outcome.failed > 0 {
                    events::LEVEL_WARN
                } else {
                    events::LEVEL_INFO
                },
                events::SOURCE_ROTATION,
                "rotation_subscription_refresh",
                format!(
                    "pre-rotation refresh: {} attempted, {} ok, {} failed",
                    outcome.attempted, outcome.succeeded, outcome.failed
                ),
                None,
                None,
                None,
            )
            .await;
        }
    }

    async fn collect_passing_rotation_candidates(
        &self,
        request: &ReplaceRequest,
        eligible_ids: &[i64],
        passing: &mut Vec<(i64, i64, Option<f64>)>,
    ) -> crate::app::Result<()> {
        if !matches!(request.trigger, RotationTrigger::Manual) {
            let fresh_results = run_rotation_bulk_tests(self.context, eligible_ids).await?;
            for row in &fresh_results {
                if !row.real_delay_ok {
                    continue;
                }
                let Some(real_delay_ms) = row.real_delay_ms else {
                    continue;
                };
                passing.push((row.id, real_delay_ms as i64, row.download_mbps));
            }
            if !passing.is_empty() {
                return Ok(());
            }
        }

        for config_id in eligible_ids {
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

        Ok(())
    }
}

fn no_passing_candidate_error() -> AppError {
    AppError::InvalidArgument(
        "no eligible replacement candidate: no enabled config has a passing real-delay result; \
         run `xrat test` or `xrat scan` first"
            .to_string(),
    )
}

fn compare_optional_f64_desc(left: Option<f64>, right: Option<f64>) -> std::cmp::Ordering {
    match (left, right) {
        (Some(a), Some(b)) => b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}
