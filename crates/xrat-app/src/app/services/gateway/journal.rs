use std::{io, net::Ipv4Addr, path::Path};

use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "gateway-ownership-v1.json";

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct InstallIntent {
    pub installation_id: uuid::Uuid,
    pub lan_interface: String,
    pub lan_ifindex: u32,
    pub lan_address: Ipv4Addr,
    pub lan_subnet: String,
    pub local_subnets: Vec<String>,
    pub previous_ipv4_forwarding: bool,
    #[serde(default)]
    pub boot_id: Option<uuid::Uuid>,
}

pub(super) fn load(directory: &Path) -> io::Result<Option<InstallIntent>> {
    match std::fs::read(directory.join(FILE_NAME)) {
        Ok(contents) => Ok(Some(serde_json::from_slice(&contents)?)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(any(target_os = "linux", test))]
pub(super) fn create(directory: &Path, intent: &InstallIntent) -> io::Result<()> {
    create_record(directory, FILE_NAME, intent)
}

#[cfg(any(target_os = "linux", test))]
pub(super) fn create_record(
    directory: &Path,
    name: &str,
    record: &impl Serialize,
) -> io::Result<()> {
    write_record(directory, name, record, false)
}

#[cfg(target_os = "linux")]
pub(super) fn replace(directory: &Path, intent: &InstallIntent) -> io::Result<()> {
    write_record(directory, FILE_NAME, intent, true)
}

#[cfg(any(target_os = "linux", test))]
fn write_record(
    directory: &Path,
    name: &str,
    record: &impl Serialize,
    replace: bool,
) -> io::Result<()> {
    use std::io::Write;

    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    serde_json::to_writer(&mut temporary, record)?;
    temporary.flush()?;
    temporary.as_file().sync_all()?;
    let path = directory.join(name);
    if replace {
        temporary.persist(path)
    } else {
        temporary.persist_noclobber(path)
    }
    .map_err(|error| error.error)?;
    std::fs::File::open(directory)?.sync_all()
}

#[cfg(target_os = "linux")]
pub(super) fn clear(directory: &Path) -> io::Result<()> {
    std::fs::remove_file(directory.join(FILE_NAME))?;
    std::fs::File::open(directory)?.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persists_intent_without_replacing_existing_record() {
        let directory = tempfile::tempdir().unwrap();
        let intent = InstallIntent {
            installation_id: uuid::Uuid::new_v4(),
            lan_interface: "eth0".into(),
            lan_ifindex: 2,
            lan_address: Ipv4Addr::new(192, 0, 2, 2),
            lan_subnet: "192.0.2.0/24".into(),
            local_subnets: Vec::new(),
            previous_ipv4_forwarding: false,
            boot_id: None,
        };
        assert_eq!(load(directory.path()).unwrap(), None);
        create(directory.path(), &intent).unwrap();
        let original_id = intent.installation_id;
        let replacement = InstallIntent {
            installation_id: uuid::Uuid::new_v4(),
            ..intent
        };
        assert_eq!(
            create(directory.path(), &replacement).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(
            load(directory.path()).unwrap().unwrap().installation_id,
            original_id
        );
    }

    #[test]
    fn malformed_intent_is_an_error() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join(FILE_NAME), b"{").unwrap();
        assert!(load(directory.path()).is_err());
    }
}
