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
mod tests {
    use super::NodeDedupKey;
    use crate::model::Protocol;

    #[test]
    fn formats_as_versioned_length_prefixed_key() {
        let key = NodeDedupKey {
            protocol: Protocol::Vless,
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: Some("uuid|123".to_string()),
            password: None,
            method: None,
            network: "ws".to_string(),
            tls: Some("tls".to_string()),
            sni: Some("cdn.example.com".to_string()),
            host: Some("cdn.example.com".to_string()),
            path: Some("/ray".to_string()),
        };

        assert_eq!(
            key.to_string(),
            "v1|protocol=5:vless|address=11:example.com|port=3:443|username=-|uuid=8:uuid|123|password=-|method=-|network=2:ws|tls=3:tls|sni=15:cdn.example.com|host=15:cdn.example.com|path=4:/ray"
        );
    }

    #[test]
    fn distinguishes_none_from_empty_string() {
        let none_key = NodeDedupKey {
            protocol: Protocol::Ss,
            address: "example.com".to_string(),
            port: 8388,
            username: None,
            uuid: None,
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: None,
            sni: None,
            host: None,
            path: None,
        };
        let empty_key = NodeDedupKey {
            password: Some(String::new()),
            ..none_key.clone()
        };

        assert_ne!(none_key.to_string(), empty_key.to_string());
        assert!(empty_key.to_string().contains("|password=0:"));
    }
}
