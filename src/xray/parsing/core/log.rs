use serde::{Deserialize, Serialize};

use crate::xray::parsing::shared::{LogLevel, MaskAddress};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loglevel: Option<LogLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_log: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask_address: Option<MaskAddress>,
}
