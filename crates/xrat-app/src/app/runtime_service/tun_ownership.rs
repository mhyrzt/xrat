use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TunOwnershipRecord {
    pub interface_name: String,
    pub ifindex: Option<u32>,
    #[serde(default)]
    pub interface_identity: Option<String>,
    pub session_id: i64,
    pub engine: String,
    #[serde(default)]
    pub policy_rules: Vec<xrat_support::net::KernelPolicyRule>,
    #[serde(default)]
    pub kill_switch_active: bool,
    #[serde(default)]
    pub kill_switch_backend: Option<String>,
}

pub fn ownership_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("tun-ownership.json")
}

pub fn load_ownership(runtime_dir: &Path) -> Option<TunOwnershipRecord> {
    let path = ownership_path(runtime_dir);
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub fn save_ownership(runtime_dir: &Path, record: &TunOwnershipRecord) -> std::io::Result<()> {
    use std::io::Write;
    std::fs::create_dir_all(runtime_dir)?;
    let path = ownership_path(runtime_dir);
    let temporary = runtime_dir.join(format!(".tun-ownership-{}.tmp", uuid::Uuid::new_v4()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| {
        let mut file = options.open(&temporary)?;
        file.write_all(&serde_json::to_vec_pretty(record)?)?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temporary);
    }
    result
}

pub fn clear_ownership(runtime_dir: &Path) {
    let path = ownership_path(runtime_dir);
    let _ = std::fs::remove_file(path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_ownership_record() {
        let temp = tempfile::tempdir().expect("temp dir");
        let runtime_dir = temp.path();

        assert_eq!(load_ownership(runtime_dir), None);

        let record = TunOwnershipRecord {
            interface_name: "xrat0".to_string(),
            ifindex: Some(42),
            session_id: 100,
            engine: "xray".to_string(),
            interface_identity: Some("{verified-platform-guid}".into()),
            policy_rules: vec![],
            kill_switch_active: false,
            kill_switch_backend: None,
        };
        save_ownership(runtime_dir, &record).expect("save");
        assert_eq!(load_ownership(runtime_dir), Some(record));

        clear_ownership(runtime_dir);
        assert_eq!(load_ownership(runtime_dir), None);
    }

    #[test]
    fn legacy_journal_has_no_platform_identity() {
        let record: TunOwnershipRecord = serde_json::from_str(r#"{"interface_name":"xrat0","ifindex":42,"session_id":100,"engine":"xray","policy_rules":[]}"#).unwrap();
        assert_eq!(record.interface_identity, None);
    }
}
