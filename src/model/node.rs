use serde::Serialize;

use crate::model::{NodeDedupKey, Protocol};

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
