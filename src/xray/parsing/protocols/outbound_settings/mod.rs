mod basic;
mod proxy;
mod tunnel;

pub use basic::{
    BlackholeResponse, OutboundSettingsBlackhole, OutboundSettingsDns, OutboundSettingsFreedom,
};
pub use proxy::{OutboundSettingsHttp, OutboundSettingsShadowsocks, OutboundSettingsSocks};
pub use tunnel::{
    OutboundSettingsHysteria, OutboundSettingsLoopback, OutboundSettingsTrojan,
    OutboundSettingsVless, OutboundSettingsVmess, OutboundSettingsWireguard,
};
