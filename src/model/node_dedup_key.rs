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
