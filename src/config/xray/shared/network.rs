use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    #[serde(rename = "tcp")]
    Tcp,
    #[serde(rename = "udp")]
    Udp,
    #[serde(rename = "tcp,udp")]
    TcpUdp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StreamNetwork {
    #[serde(rename = "raw")]
    Raw,
    #[serde(rename = "xhttp")]
    Xhttp,
    #[serde(rename = "kcp")]
    Kcp,
    #[serde(rename = "grpc")]
    Grpc,
    #[serde(rename = "ws")]
    Ws,
    #[serde(rename = "httpupgrade")]
    HttpUpgrade,
    #[serde(rename = "hysteria")]
    Hysteria,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Security {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "tls")]
    Tls,
    #[serde(rename = "reality")]
    Reality,
}
