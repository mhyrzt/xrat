use xrat_model::ConfigId;

pub const XRAY_PROFILE_PROTOCOL: &str = "xray-json";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XrayProfileRecord {
    pub config_id: ConfigId,
    pub digest: String,
    pub byte_length: i64,
}
