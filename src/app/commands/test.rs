mod bulk;
mod execution;
mod handlers;
mod output;
mod output_types;
mod settings;
mod stages;

use std::cmp::Ordering;
use std::io::Write;
use std::net::IpAddr;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;
use tokio::task::JoinSet;

use crate::app::AppError;
use crate::app::config::defaults;
use crate::app::config::{AppConfig, ConnectionTestStage, TestFailurePolicy};
use crate::app::context::{AppContext, RuntimePaths};
use crate::cli::{TestArgs, TestFormat, TestSortBy};
#[cfg(test)]
use crate::db::DatabaseConnectionConfig;
use crate::db::{ConfigRecord, ConnectionTestInsert, ConnectionTestRunInsert, Database};
use crate::model::Node;
use crate::prober::{
    FailureKind, TestResult, download_speed_check, icmp_ping, real_delay_check, tcp_check,
    upload_speed_check,
};
use crate::{app::config, support::geoip};

pub(crate) use bulk::run_rotation_bulk_tests;
use bulk::*;
use execution::*;
pub use handlers::run;
use output::*;
use output_types::*;
use settings::*;
use stages::*;

#[cfg(test)]
mod tests;

pub(crate) async fn run_bulk_for_config_ids_cancellable(
    args: &TestArgs,
    context: &AppContext,
    config_ids: &[i64],
    cancel_rx: crate::support::cancel::CancellationReceiver,
) -> crate::app::Result<usize> {
    let settings = resolve_test_settings(args, &context.app_config, &context.runtime_paths)?;
    let mut configs = Vec::with_capacity(config_ids.len());

    for config_id in config_ids {
        if let Some(config) = context.db.get_config_by_id(*config_id).await? {
            configs.push(config);
        }
    }

    if configs.is_empty() {
        return Ok(0);
    }

    let rows = bulk::run_bulk_for_configs_cancellable(
        context,
        settings,
        configs,
        "tui",
        false,
        Some(cancel_rx),
    )
    .await?;
    Ok(rows.len())
}
