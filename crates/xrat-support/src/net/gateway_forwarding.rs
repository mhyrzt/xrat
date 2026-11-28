use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

use super::{SystemTunInterfaceOps, TunInterfaceOps};

#[derive(Debug, Serialize, Deserialize)]
pub struct InterfaceForwarding {
    interface: String,
    ifindex: u32,
    previous: bool,
}

impl InterfaceForwarding {
    pub fn inspect(interface: &str) -> io::Result<Self> {
        let path = forwarding_path(interface)?;
        let identity = SystemTunInterfaceOps
            .inspect_interface(interface)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "interface is missing"))?;
        Ok(Self {
            interface: interface.into(),
            ifindex: identity.ifindex,
            previous: read_flag(&path)?,
        })
    }

    pub fn enable(&self) -> io::Result<()> {
        let path = self.verify_interface()?;
        if !read_flag(&path)? {
            write_flag(&path, true)?;
        }
        Ok(())
    }

    pub fn restore(&self) -> io::Result<()> {
        let path = self.verify_interface()?;
        if !self.previous && read_flag(&path)? {
            write_flag(&path, false)?;
        }
        Ok(())
    }

    fn verify_interface(&self) -> io::Result<PathBuf> {
        let path = forwarding_path(&self.interface)?;
        let interface = SystemTunInterfaceOps.inspect_interface(&self.interface)?;
        if interface.is_none_or(|interface| interface.ifindex != self.ifindex) {
            return Err(io::Error::other(
                "gateway forwarding interface identity changed",
            ));
        }
        Ok(path)
    }
}

fn forwarding_path(interface: &str) -> io::Result<PathBuf> {
    if interface.is_empty()
        || matches!(interface, "." | ".." | "all" | "default")
        || interface.contains('/')
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a specific Linux network interface",
        ));
    }
    Ok(PathBuf::from("/proc/sys/net/ipv4/conf")
        .join(interface)
        .join("forwarding"))
}

fn read_flag(path: &std::path::Path) -> io::Result<bool> {
    match fs::read_to_string(path)?.trim() {
        "0" => Ok(false),
        "1" => Ok(true),
        value => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unexpected forwarding value {value:?}"),
        )),
    }
}

fn write_flag(path: &std::path::Path, enabled: bool) -> io::Result<()> {
    fs::write(path, if enabled { "1" } else { "0" })?;
    if read_flag(path)? != enabled {
        return Err(io::Error::other("IPv4 forwarding read-back differed"));
    }
    Ok(())
}
