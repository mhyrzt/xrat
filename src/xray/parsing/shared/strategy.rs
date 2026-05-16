use serde::{Deserialize, Serialize};

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
