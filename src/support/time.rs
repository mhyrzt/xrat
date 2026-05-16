use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub fn now_string() -> String {
    now_epoch_seconds().to_string()
}
