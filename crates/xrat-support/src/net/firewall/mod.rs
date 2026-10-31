use std::io;
#[cfg(target_os = "linux")]
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

mod types;
pub use types::KillSwitchConfig;

#[cfg(target_os = "linux")]
pub mod iptables;
#[cfg(target_os = "linux")]
pub mod nftables;

#[cfg(test)]
mod tests;

#[cfg(target_os = "linux")]
pub(crate) fn find_in_path(binary: &str) -> Option<PathBuf> {
    let search_path = std::env::var_os("PATH")?;
    std::env::split_paths(&search_path)
        .map(|directory| directory.join(binary))
        .find(|candidate| candidate.is_file())
}

/// Operations for managing a kernel-level fail-closed kill switch.
pub trait FirewallOps: Send + Sync {
    /// Apply the kill switch rules.
    fn apply_killswitch(&self, config: &KillSwitchConfig) -> io::Result<()>;
    /// Remove any active kill switch rules.
    fn remove_killswitch(&self) -> io::Result<()>;
    /// Query whether kill switch rules are currently active in the kernel.
    fn is_active(&self) -> io::Result<bool>;
    /// Name of the underlying firewall backend.
    fn backend_name(&self) -> &'static str;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemFirewallOps;

impl FirewallOps for SystemFirewallOps {
    fn apply_killswitch(&self, config: &KillSwitchConfig) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            if nftables::is_available() {
                return nftables::apply(config);
            }
            if iptables::is_available() {
                return iptables::apply(config);
            }
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "neither nft nor iptables is available for kill switch",
            ))
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = config;
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "kill switch firewall operations are only supported on Linux currently",
            ))
        }
    }

    fn remove_killswitch(&self) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            let mut err = None;
            #[allow(clippy::collapsible_if)]
            if nftables::is_available() {
                if let Err(e) = nftables::remove() {
                    err = Some(e);
                }
            }
            #[allow(clippy::collapsible_if)]
            if find_in_path("iptables").is_some() {
                if let Err(e) = iptables::remove() {
                    if err.is_none() {
                        err = Some(e);
                    }
                }
            }
            match err {
                Some(e) => Err(e),
                None => Ok(()),
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            Ok(())
        }
    }

    fn is_active(&self) -> io::Result<bool> {
        #[cfg(target_os = "linux")]
        {
            if nftables::is_available() && nftables::is_active().unwrap_or(false) {
                return Ok(true);
            }
            if iptables::is_available() && iptables::is_active().unwrap_or(false) {
                return Ok(true);
            }
            Ok(false)
        }
        #[cfg(not(target_os = "linux"))]
        {
            Ok(false)
        }
    }

    fn backend_name(&self) -> &'static str {
        #[cfg(target_os = "linux")]
        {
            if nftables::is_available() {
                "nftables"
            } else if iptables::is_available() {
                "iptables"
            } else {
                "none"
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            "unsupported"
        }
    }
}

/// In-memory mock firewall operations for deterministic testing.
#[derive(Debug, Default)]
pub struct MockFirewallOps {
    pub active: AtomicBool,
    pub fail_apply: AtomicBool,
    pub fail_remove: AtomicBool,
    pub last_config: Mutex<Option<KillSwitchConfig>>,
}

impl FirewallOps for MockFirewallOps {
    fn apply_killswitch(&self, config: &KillSwitchConfig) -> io::Result<()> {
        if self.fail_apply.load(Ordering::SeqCst) {
            return Err(io::Error::other("mock firewall apply failure"));
        }
        self.active.store(true, Ordering::SeqCst);
        let mut guard = self.last_config.lock().unwrap();
        *guard = Some(config.clone());
        Ok(())
    }

    fn remove_killswitch(&self) -> io::Result<()> {
        if self.fail_remove.load(Ordering::SeqCst) {
            return Err(io::Error::other("mock firewall remove failure"));
        }
        self.active.store(false, Ordering::SeqCst);
        let mut guard = self.last_config.lock().unwrap();
        *guard = None;
        Ok(())
    }

    fn is_active(&self) -> io::Result<bool> {
        Ok(self.active.load(Ordering::SeqCst))
    }

    fn backend_name(&self) -> &'static str {
        "mock"
    }
}
