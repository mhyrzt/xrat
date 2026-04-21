use serde::Serialize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Vless,
    Vmess,
    Ss,
    Trojan,
    Http,
    Socks5,
}

impl Protocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Vless => "vless",
            Self::Vmess => "vmess",
            Self::Ss => "ss",
            Self::Trojan => "trojan",
            Self::Http => "http",
            Self::Socks5 => "socks5",
        }
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Node {
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
    pub name: Option<String>,
}

impl Node {
    pub fn dedup_key(&self) -> NodeDedupKey {
        NodeDedupKey {
            protocol: self.protocol.clone(),
            address: self.address.clone(),
            port: self.port,
            username: self.username.clone(),
            uuid: self.uuid.clone(),
            password: self.password.clone(),
        }
    }

    pub fn dedup_key_string(&self) -> String {
        self.dedup_key().to_string()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeDedupKey {
    pub protocol: Protocol,
    pub address: String,
    pub port: u16,
    pub username: Option<String>,
    pub uuid: Option<String>,
    pub password: Option<String>,
}

impl fmt::Display for NodeDedupKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|{}",
            self.protocol,
            self.address,
            self.port,
            self.username.as_deref().unwrap_or_default(),
            self.uuid.as_deref().unwrap_or_default(),
            self.password.as_deref().unwrap_or_default()
        )
    }
}
