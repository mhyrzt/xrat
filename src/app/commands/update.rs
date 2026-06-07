use std::collections::HashMap;

use crate::app::commands::output;
use crate::app::commands::resolve::resolve_subscription_id;
use crate::app::context::AppContext;
use crate::app::import;
use crate::cli::UpdateArgs;
use crate::db::SubscriptionRecord;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionUpdateResult {
    pub subscription_id: i64,
    pub subscription_ref: String,
    pub subscription_name: String,
    pub imported_configs: usize,
    pub removed_configs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionUpdateError {
    pub subscription_id: i64,
    pub subscription_ref: String,
    pub subscription_name: String,
    pub error: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SubscriptionUpdateSummary {
    pub attempted: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub imported_configs: usize,
    pub removed_configs: u64,
    pub successes: Vec<SubscriptionUpdateResult>,
    pub failures: Vec<SubscriptionUpdateError>,
}

impl SubscriptionUpdateSummary {
    pub fn status_message(&self) -> String {
        if self.attempted == 0 {
            return "No subscriptions matched.".to_string();
        }
        if self.failed == 0 {
            if self.succeeded == 1 {
                let item = &self.successes[0];
                return format!("{} updated", item.subscription_name);
            }
            return "All subscriptions updated!".to_string();
        }
        format!("{} of {} subscriptions failed", self.failed, self.attempted)
    }
}

pub async fn run(context: &AppContext, args: &UpdateArgs) -> crate::app::Result<()> {
    let summary = if args.subs_ref.is_empty() {
        update_all(context).await?
    } else {
        update_by_refs(context, &args.subs_ref).await?
    };

    print_summary(&summary);

    Ok(())
}

pub async fn update_by_ids(
    context: &AppContext,
    ids: &[i64],
) -> crate::app::Result<SubscriptionUpdateSummary> {
    let mut targets = Vec::with_capacity(ids.len());
    for id in ids {
        if let Some(subscription) = context.db.get_subscription_by_id(*id).await? {
            targets.push(subscription);
        }
    }
    update_targets(context, targets).await
}

pub async fn update_all(context: &AppContext) -> crate::app::Result<SubscriptionUpdateSummary> {
    let subscriptions = context.db.list_subscriptions().await?;
    let targets = subscriptions
        .into_iter()
        .filter(|subscription| !subscription.source_url.as_deref().unwrap_or("").is_empty())
        .collect::<Vec<_>>();
    update_targets(context, targets).await
}

pub async fn update_by_refs(
    context: &AppContext,
    refs: &[String],
) -> crate::app::Result<SubscriptionUpdateSummary> {
    let mut targets = Vec::with_capacity(refs.len());
    for raw in refs {
        let id = resolve_subscription_id(context, raw).await?;
        let subscription = context
            .db
            .get_subscription_by_id(id)
            .await?
            .ok_or_else(|| {
                crate::app::AppError::InvalidArgument(format!("no subscription found for '{raw}'"))
            })?;
        targets.push(subscription);
    }
    update_targets(context, targets).await
}

async fn update_targets(
    context: &AppContext,
    targets: Vec<SubscriptionRecord>,
) -> crate::app::Result<SubscriptionUpdateSummary> {
    let mut summary = SubscriptionUpdateSummary::default();
    let mut dedup = HashMap::<i64, SubscriptionRecord>::new();
    for subscription in targets {
        dedup.insert(subscription.id, subscription);
    }
    let mut ordered = dedup.into_values().collect::<Vec<_>>();
    ordered.sort_by_key(|subscription| subscription.id);

    for subscription in ordered {
        let Some(source_url) = subscription.source_url.as_deref() else {
            continue;
        };
        if source_url.is_empty() {
            continue;
        }
        summary.attempted += 1;
        let label = subscription
            .name
            .as_deref()
            .filter(|value| !value.is_empty())
            .unwrap_or(subscription.r#ref.as_str())
            .to_string();
        match refresh_one(context, source_url).await {
            Ok(import_summary) => {
                summary.succeeded += 1;
                summary.imported_configs += import_summary.imported_configs;
                summary.removed_configs += import_summary.removed_configs;
                summary.successes.push(SubscriptionUpdateResult {
                    subscription_id: subscription.id,
                    subscription_ref: subscription.r#ref.clone(),
                    subscription_name: label,
                    imported_configs: import_summary.imported_configs,
                    removed_configs: import_summary.removed_configs,
                });
            }
            Err(error) => {
                summary.failed += 1;
                summary.failures.push(SubscriptionUpdateError {
                    subscription_id: subscription.id,
                    subscription_ref: subscription.r#ref.clone(),
                    subscription_name: label,
                    error: error.to_string(),
                });
            }
        }
    }

    Ok(summary)
}

async fn refresh_one(
    context: &AppContext,
    source_url: &str,
) -> crate::app::Result<crate::db::ImportSummary> {
    let (source, nodes) = import::load_nodes_async(source_url).await?;
    Ok(context.db.import_nodes(&source, &nodes).await?)
}

fn print_summary(summary: &SubscriptionUpdateSummary) {
    println!(
        "{}",
        output::notice(summary.status_message(), output::color_enabled())
    );
    println!(
        "{}",
        output::format_kv(
            Some("Subscription update"),
            &[
                ("attempted", summary.attempted.to_string()),
                ("succeeded", summary.succeeded.to_string()),
                ("failed", summary.failed.to_string()),
                ("imported configs", summary.imported_configs.to_string()),
                ("removed configs", summary.removed_configs.to_string()),
            ],
            output::color_enabled(),
        )
    );

    if summary.failed > 0 {
        let failed = summary
            .failures
            .iter()
            .map(|entry| {
                format!(
                    "{} ({}) — {}",
                    entry.subscription_name, entry.subscription_ref, entry.error
                )
            })
            .collect::<Vec<_>>();
        println!(
            "{}",
            output::format_list("Failed subscriptions", &failed, output::color_enabled())
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_message_reports_all_updated_for_multi_success() {
        let summary = SubscriptionUpdateSummary {
            attempted: 2,
            succeeded: 2,
            failed: 0,
            imported_configs: 3,
            removed_configs: 1,
            successes: vec![
                SubscriptionUpdateResult {
                    subscription_id: 1,
                    subscription_ref: "aabbccdd".to_string(),
                    subscription_name: "main".to_string(),
                    imported_configs: 2,
                    removed_configs: 1,
                },
                SubscriptionUpdateResult {
                    subscription_id: 2,
                    subscription_ref: "eeff0011".to_string(),
                    subscription_name: "backup".to_string(),
                    imported_configs: 1,
                    removed_configs: 0,
                },
            ],
            failures: Vec::new(),
        };

        assert_eq!(summary.status_message(), "All subscriptions updated!");
    }

    #[test]
    fn status_message_reports_named_single_success() {
        let summary = SubscriptionUpdateSummary {
            attempted: 1,
            succeeded: 1,
            failed: 0,
            imported_configs: 1,
            removed_configs: 0,
            successes: vec![SubscriptionUpdateResult {
                subscription_id: 1,
                subscription_ref: "aabbccdd".to_string(),
                subscription_name: "main".to_string(),
                imported_configs: 1,
                removed_configs: 0,
            }],
            failures: Vec::new(),
        };

        assert_eq!(summary.status_message(), "main updated");
    }

    #[test]
    fn status_message_reports_failures() {
        let summary = SubscriptionUpdateSummary {
            attempted: 4,
            succeeded: 2,
            failed: 2,
            imported_configs: 3,
            removed_configs: 0,
            successes: Vec::new(),
            failures: Vec::new(),
        };
        assert_eq!(summary.status_message(), "2 of 4 subscriptions failed");
    }
}
