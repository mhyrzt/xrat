use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::model::{NodeDedupKey, Protocol};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
    pub raw_config: String,
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
            method: self.method.clone(),
            network: self.network.clone(),
            tls: self.tls.clone(),
            sni: self.sni.clone(),
            host: self.host.clone(),
            path: self.path.clone(),
            extensions: self.extensions.clone(),
        }
    }

    pub fn dedup_key_string(&self) -> String {
        self.dedup_key().to_string()
    }

    pub fn extension_value(&self, key: &str) -> Option<&serde_json::Value> {
        self.extensions.as_ref()?.get(key)
    }

    pub fn extension_string(&self, key: &str) -> Option<String> {
        match self.extension_value(key)? {
            serde_json::Value::String(value) => Some(value.clone()),
            serde_json::Value::Bool(value) => Some(value.to_string()),
            serde_json::Value::Number(value) => Some(value.to_string()),
            _ => None,
        }
    }
}
