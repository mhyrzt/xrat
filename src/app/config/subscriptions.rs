use serde::{Deserialize, Serialize};

use super::defaults;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct SubscriptionSettings {
    pub auto_refresh: bool,
    pub refresh_interval_hours: u64,
}

impl Default for SubscriptionSettings {
    fn default() -> Self {
        Self {
            auto_refresh: defaults::DEFAULT_SUBSCRIPTIONS_AUTO_REFRESH,
            refresh_interval_hours: defaults::DEFAULT_SUBSCRIPTIONS_REFRESH_INTERVAL_HOURS,
        }
    }
}

impl SubscriptionSettings {
    /// Refresh interval in seconds, clamped so a misconfigured `0` does not turn
    /// the scheduler into a busy loop.
    pub fn refresh_interval_secs(&self) -> u64 {
        self.refresh_interval_hours.max(1) * 3600
    }
}
