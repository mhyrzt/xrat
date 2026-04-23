use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Address = String;
pub type Cidr = String;
pub type DomainMatcher = String;
pub type DurationString = String;
pub type StringMap = HashMap<String, String>;
pub type StringArrayMap = HashMap<String, Vec<String>>;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MaskAddress {
    #[serde(rename = "")]
    Empty,
    #[serde(rename = "quarter")]
    Quarter,
    #[serde(rename = "half")]
    Half,
    #[serde(rename = "full")]
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QueryStrategy {
    UseIP,
    UseIPv4,
    UseIPv6,
    UseSystem,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DomainStrategy {
    AsIs,
    UseIP,
    UseIPv4,
    UseIPv6,
    UseIPv4v6,
    UseIPv6v4,
    ForceIP,
    ForceIPv4,
    ForceIPv6,
    ForceIPv4v6,
    ForceIPv6v4,
}

/// Port value can be a number or a string range like "1000-2000"
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum PortValue {
    Single(u16),
    Range(String),
}

/// Int32 range can be a number or a string range
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Int32Range {
    Single(i32),
    Range(String),
}
