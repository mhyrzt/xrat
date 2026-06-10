use std::sync::Arc;

use tokio::sync::mpsc;

use crate::app::context::AppContext;
use crate::support::geoip::CachedLookup;
use crate::tui::app::TuiApp;
use crate::tui::data::{TuiData, TuiLogs};
use crate::tui::task::{TuiTaskEvent, TuiTaskKind};

/// Bound on concurrent address resolutions so enrichment does not fan out one
/// DNS query per config serially.
const ENRICH_CONCURRENCY: usize = 64;

/// Per-address cap so an unresolvable or slow host cannot stall enrichment.
const ENRICH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

/// Number of resolved rows accumulated before flushing a batch back to the UI so
/// locations fill in progressively instead of all at once.
const ENRICH_FLUSH_BATCH: usize = 16;

/// Resolve missing endpoint locations off the critical path and feed results
/// back through the task channel so the first frame never waits on DNS.
pub fn spawn_enrich_locations(
    lookup: Arc<CachedLookup>,
    targets: Vec<(i64, String)>,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    if targets.is_empty() {
        return;
    }

    let task_tx = task_tx.clone();
    tokio::spawn(async move {
        let mut join_set = tokio::task::JoinSet::new();
        let mut pending = targets.into_iter();
        let spawn_next =
            |join_set: &mut tokio::task::JoinSet<_>,
             pending: &mut std::vec::IntoIter<(i64, String)>| {
                if let Some((id, address)) = pending.next() {
                    let lookup = lookup.clone();
                    join_set.spawn(async move {
                        let meta = tokio::time::timeout(
                            ENRICH_TIMEOUT,
                            crate::support::geoip::enrich_address(&address, lookup.as_ref()),
                        )
                        .await
                        .unwrap_or_default();
                        (id, meta)
                    });
                }
            };

        for _ in 0..ENRICH_CONCURRENCY {
            spawn_next(&mut join_set, &mut pending);
        }

        let mut batch = Vec::new();
        while let Some(joined) = join_set.join_next().await {
            if let Ok((id, meta)) = joined
                && meta.has_lookup_metadata()
            {
                batch.push((id, meta));
                if batch.len() >= ENRICH_FLUSH_BATCH {
                    let _ = task_tx.send(TuiTaskEvent::LocationsEnriched {
                        updates: std::mem::take(&mut batch),
                    });
                }
            }
            spawn_next(&mut join_set, &mut pending);
        }

        if !batch.is_empty() {
            let _ = task_tx.send(TuiTaskEvent::LocationsEnriched { updates: batch });
        }
    });
}

/// Collect the `(id, address)` pairs that still need location enrichment.
pub fn enrichment_targets(data: &TuiData) -> Vec<(i64, String)> {
    data.configs
        .iter()
        .filter(|config| config.needs_location_enrichment())
        .map(|config| (config.id, config.address.clone()))
        .collect()
}

pub fn spawn_reload_data(
    context: AppContext,
    include_deleted: bool,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    let kind = TuiTaskKind::ReloadData;
    let _ = task_tx.send(TuiTaskEvent::Started { kind });

    let task_tx = task_tx.clone();
    tokio::spawn(async move {
        let event = match TuiData::load(&context, include_deleted).await {
            Ok(data) => TuiTaskEvent::Completed {
                kind,
                message: "reloaded data".to_string(),
                data: Some(data),
            },
            Err(error) => TuiTaskEvent::Failed {
                kind,
                error: error.to_string(),
                data: None,
            },
        };
        let _ = task_tx.send(event);
    });
}

pub async fn reload_data(context: &AppContext, app: &mut TuiApp) {
    match TuiData::load(context, app.config_list.include_deleted).await {
        Ok(data) => app.reload_data(data),
        Err(error) => app.push_log(format!("ERR reload failed: {error}")),
    }
}

pub fn spawn_reload_logs(
    context: AppContext,
    logs_tx: &mpsc::UnboundedSender<crate::app::Result<TuiLogs>>,
) {
    let logs_tx = logs_tx.clone();
    tokio::spawn(async move {
        let _ = logs_tx.send(TuiLogs::load(&context).await);
    });
}

async fn reload_logs(context: &AppContext, app: &mut TuiApp) {
    match TuiLogs::load(context).await {
        Ok(logs) => app.reload_logs(logs),
        Err(error) => app.push_log(format!("ERR log reload failed: {error}")),
    }
}

/// Clear all persisted events from the database, then refresh the logs card.
/// This is the destructive DB clear; engine log files are left untouched.
pub async fn run_clear_events(context: &AppContext, app: &mut TuiApp) {
    match context.db.clear_events().await {
        Ok(count) => {
            reload_logs(context, app).await;
            app.push_log(format!("OK  cleared {count} persisted event(s) from db"));
        }
        Err(error) => app.push_log(format!("ERR clear events failed: {error}")),
    }
}
