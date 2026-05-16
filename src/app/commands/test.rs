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
