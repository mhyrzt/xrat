use super::*;
use std::os::unix::fs::PermissionsExt;

struct Fixture {
    directory: tempfile::TempDir,
    binary: String,
}

impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("iptables");
        std::fs::write(&path, include_str!("iptables_test_driver.sh")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        Self {
            binary: path.to_str().unwrap().to_string(),
            directory,
        }
    }

    fn mark(&self, name: &str) {
        std::fs::write(self.directory.path().join(name), "").unwrap();
    }

    fn exists(&self, name: &str) -> bool {
        self.directory.path().join(name).exists()
    }

    fn attach(&self, chain: &str) {
        for suffix in ["", ".drop", ".attached"] {
            self.mark(&format!("{chain}{suffix}"));
        }
    }
}

#[test]
fn interrupted_swap_recovery_preserves_protection_on_failure_and_retry() {
    for is_v4 in [true, false] {
        for old_attached in [true, false] {
            for failure in ["fail-N", "fail-E"] {
                let fixture = Fixture::new();
                fixture.attach(CHAIN_STAGING);
                if old_attached {
                    fixture.attach(CHAIN_NAME);
                }
                fixture.mark("guard");
                fixture.mark(failure);
                let config = KillSwitchConfig::new("xrat0");
                assert!(apply_staging(&fixture.binary, &config, is_v4).is_err());
                assert!(!fixture.exists("leaked"));
                assert!(has_active_chain(&fixture.binary).unwrap());

                std::fs::remove_file(fixture.directory.path().join(failure)).unwrap();
                apply_staging(&fixture.binary, &config, is_v4).unwrap();
                assert!(fixture.exists("XRAT_KS.attached"));
                assert!(!fixture.exists("XRAT_KS_STG.attached"));
                assert!(!fixture.exists("leaked"));
            }
        }
    }
}

#[test]
fn removal_reports_failure_and_can_be_retried() {
    let fixture = Fixture::new();
    fixture.attach(CHAIN_NAME);
    fixture.mark("fail-D");
    assert!(cleanup_chain(&fixture.binary, CHAIN_NAME).is_err());
    assert!(has_active_chain(&fixture.binary).unwrap());
    std::fs::remove_file(fixture.directory.path().join("fail-D")).unwrap();
    cleanup_chain(&fixture.binary, CHAIN_NAME).unwrap();
    cleanup_chain(&fixture.binary, CHAIN_NAME).unwrap();
    assert!(!has_active_chain(&fixture.binary).unwrap());
}

#[test]
fn firewall_queries_propagate_permission_errors() {
    let fixture = Fixture::new();
    assert!(!has_active_chain(&fixture.binary).unwrap());
    fixture.attach(CHAIN_STAGING);
    assert!(has_active_chain(&fixture.binary).unwrap());
    fixture.mark("fail-query");
    assert!(check_chain_in_output(&fixture.binary, CHAIN_NAME).is_err());
    assert!(cleanup_chain(&fixture.binary, CHAIN_NAME).is_err());
}
