pub(crate) mod catalog_import;
pub mod clipboard;
mod configs;
pub mod dashboard;
#[cfg(any(windows, test))]
pub(crate) mod desktop_proxy;
pub mod diagnostics;
pub mod engine_probe;
mod lifecycle;
pub mod machine_service;
pub mod proxy_pac;
pub(crate) mod release_notes;
pub mod releases;
pub mod rotation;
pub mod runtime_control;
pub(crate) mod runtime_dashboard;
pub(crate) mod runtime_traffic;
pub mod runtime_transitions;
pub mod runtime_tuning;
pub mod split_tunnel;
pub(crate) mod subscriptions;
pub mod testing;
pub(crate) mod tun;
pub(crate) mod tun_control;

pub use configs::{
    ConfigExportRequest, ConfigListRequest, ConfigListResult, ConfigService,
    DatabaseConfigRepository, MAX_TOP, enrich_endpoint_locations, validate_top,
};
pub use lifecycle::{ConfigLifecycleService, DeleteOutcome, RestoreOutcome, ToggleOutcome};

#[cfg(test)]
pub mod test_support;

use std::sync::Arc;

use crate::app::context::AppContext;
use crate::app::ports::{Clock, Filesystem, RealFilesystem, SystemClock};

mod contracts;
pub use contracts::AppServices;
pub(crate) mod xray_profiles;

pub(crate) mod cores;
pub(crate) mod web_management;

pub(crate) mod upgrades;

pub(crate) mod geoip_assets;

pub(crate) mod windows_tun;
