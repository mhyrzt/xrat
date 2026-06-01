use std::fmt;

use crate::model::Protocol;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeDedupKey {
    pub protocol: Protocol,
    pub address: String,
    pub port: u16,
    pub username: Option<String>,
    pub uuid: Option<String>,
    pub password: Option<String>,
    pub method: Option<String>,
    pub network: String,
    pub tls: Option<String>,
    pub sni: Option<String>,
    pub host: Option<String>,
    pub path: Option<String>,
}

impl fmt::Display for NodeDedupKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("v1")?;
        write_required(f, "protocol", self.protocol.as_str())?;
        write_required(f, "address", &self.address)?;
        write_required(f, "port", &self.port.to_string())?;
        write_optional(f, "username", self.username.as_deref())?;
        write_optional(f, "uuid", self.uuid.as_deref())?;
        write_optional(f, "password", self.password.as_deref())?;
        write_optional(f, "method", self.method.as_deref())?;
        write_required(f, "network", &self.network)?;
        write_optional(f, "tls", self.tls.as_deref())?;
        write_optional(f, "sni", self.sni.as_deref())?;
        write_optional(f, "host", self.host.as_deref())?;
        write_optional(f, "path", self.path.as_deref())?;
        Ok(())
    }
}

fn write_required(f: &mut fmt::Formatter<'_>, name: &str, value: &str) -> fmt::Result {
    write!(f, "|{}={}:{}", name, value.chars().count(), value)
}

fn write_optional(f: &mut fmt::Formatter<'_>, name: &str, value: Option<&str>) -> fmt::Result {
    match value {
        Some(value) => write_required(f, name, value),
        None => write!(f, "|{}=-", name),
    }
}

#[cfg(test)]
mod tests;
