//! Fire-and-forget recording of app-level events into the `events` table.
//!
//! Recording must never break the operation that produced the event, so every
//! helper here swallows database errors after logging them via `tracing`.

use crate::db::{Database, NewEvent};

pub const LEVEL_INFO: &str = "info";
pub const LEVEL_WARN: &str = "warn";
pub const LEVEL_ERROR: &str = "error";

pub const SOURCE_DAEMON: &str = "daemon";
pub const SOURCE_RUNTIME: &str = "runtime";
pub const SOURCE_ROTATION: &str = "rotation";
pub const SOURCE_HEALTH: &str = "health";
pub const SOURCE_TEST: &str = "test";
pub const SOURCE_SUBSCRIPTION: &str = "subscription";
pub const SOURCE_API: &str = "api";
pub const SOURCE_SETUP: &str = "setup";
pub const SOURCE_SETTINGS: &str = "settings";

/// Record an event, logging (but not propagating) any database failure.
#[allow(clippy::too_many_arguments)]
pub async fn record(
    db: &Database,
    level: &str,
    source: &str,
    kind: &str,
    message: impl Into<String>,
    config_id: Option<i64>,
    session_id: Option<i64>,
    detail: Option<String>,
) {
    let event = NewEvent {
        level: level.to_string(),
        source: source.to_string(),
        kind: kind.to_string(),
        config_id,
        session_id,
        message: message.into(),
        detail,
    };

    if let Err(error) = db.record_event(&event).await {
        tracing::warn!(%error, kind, "failed to record event");
    }
}
