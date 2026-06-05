//! Shared subscription refresh service.
//!
//! Re-fetches URL-backed subscriptions through the same import + reconciliation
//! path as a manual refresh, then stamps `last_refreshed_at` (done inside
//! `Database::import_nodes` for URL sources). Used by the daemon auto-refresh
//! scheduler and by pre-rotation refresh. Per-subscription failures are logged
//! as events and never abort the batch.

use crate::app::context::AppContext;
use crate::app::events;
use crate::app::import;
use crate::db::ImportSummary;
use crate::support::time::now_epoch_seconds;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RefreshOutcome {
    pub attempted: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub imported_configs: usize,
    pub removed_configs: u64,
}

/// Refresh every URL-backed subscription, ignoring the interval. Used before
/// rotation so candidate selection sees the freshest provider configs.
pub async fn refresh_all(context: &AppContext) -> RefreshOutcome {
    refresh_targets(context, i64::MAX).await
}

/// Refresh URL-backed subscriptions whose configured interval has elapsed since
/// their last successful refresh (or that were never refreshed).
pub async fn refresh_due(context: &AppContext) -> RefreshOutcome {
    let interval = context
        .app_config
        .subscriptions
        .refresh_interval_secs()
        .min(i64::MAX as u64) as i64;
    let cutoff = (now_epoch_seconds() as i64).saturating_sub(interval);
    refresh_targets(context, cutoff).await
}

async fn refresh_targets(context: &AppContext, cutoff_epoch_secs: i64) -> RefreshOutcome {
    let mut outcome = RefreshOutcome::default();
    let due = match context
        .db
        .list_refreshable_due_subscriptions(cutoff_epoch_secs)
        .await
    {
        Ok(due) => due,
        Err(error) => {
            tracing::warn!(%error, "failed to list refreshable subscriptions");
            return outcome;
        }
    };

    if due.is_empty() {
        return outcome;
    }

    events::record(
        &context.db,
        events::LEVEL_INFO,
        events::SOURCE_SUBSCRIPTION,
        "subscription_refresh_started",
        format!("refreshing {} URL-backed subscription(s)", due.len()),
        None,
        None,
        None,
    )
    .await;

    for subscription in due {
        outcome.attempted += 1;
        match refresh_one(context, &subscription.source_url).await {
            Ok(summary) => {
                outcome.succeeded += 1;
                outcome.imported_configs += summary.imported_configs;
                outcome.removed_configs += summary.removed_configs;
                events::record(
                    &context.db,
                    events::LEVEL_INFO,
                    events::SOURCE_SUBSCRIPTION,
                    "subscription_refresh_succeeded",
                    format!(
                        "refreshed subscription #{}: {} imported, {} removed",
                        subscription.id, summary.imported_configs, summary.removed_configs
                    ),
                    None,
                    None,
                    None,
                )
                .await;
            }
            Err(error) => {
                outcome.failed += 1;
                events::record(
                    &context.db,
                    events::LEVEL_WARN,
                    events::SOURCE_SUBSCRIPTION,
                    "subscription_refresh_failed",
                    format!(
                        "failed to refresh subscription #{}: {error}",
                        subscription.id
                    ),
                    None,
                    None,
                    Some(error.to_string()),
                )
                .await;
            }
        }
    }

    outcome
}

async fn refresh_one(context: &AppContext, source_url: &str) -> crate::app::Result<ImportSummary> {
    let (source, nodes) = import::load_nodes_async(source_url).await?;
    Ok(context.db.import_nodes(&source, &nodes).await?)
}
