mod bulk;
mod entrypoints;
mod execution;
mod model;
mod output;
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
use crate::app::runtime::{AppContext, RuntimePaths};
use crate::cli::{TestArgs, TestFormat, TestSortBy};
#[cfg(test)]
use crate::db::DatabaseConnectionConfig;
use crate::db::{ConfigRecord, ConnectionTestInsert, ConnectionTestRunInsert, Database};
use crate::model::Node;
use crate::tester::{
    FailureKind, TestResult, download_speed_check, icmp_ping, real_delay_check, tcp_check,
    upload_speed_check,
};
use crate::{app::config, support::geoip};

use bulk::*;
pub use entrypoints::run;
use execution::*;
use model::*;
use output::*;
use settings::*;
use stages::*;

#[cfg(test)]
mod tests;
