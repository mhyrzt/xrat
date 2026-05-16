#[cfg(test)]
use super::types::{PathBuf, SystemTime, UNIX_EPOCH};

#[cfg(test)]
pub(super) fn test_database_path(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{unique}.sqlite"))
}
