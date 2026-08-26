mod basic;
mod proxy;
mod tunnel;

pub use basic::{
    BlackholeResponse, DnsOutboundRule, FreedomFinalRule, OutboundSettingsBlackhole,
    OutboundSettingsDns, OutboundSettingsFreedom,
};
pub use proxy::{
    OutboundHttpConfig, OutboundSettingsHttp, OutboundSettingsShadowsocks, OutboundSettingsSocks,
    OutboundShadowsocksConfig, OutboundSocksConfig,
};
pub use tunnel::{
    OutboundSettingsHysteria, OutboundSettingsLoopback, OutboundSettingsWireguard,
    OutboundTrojanConfig, OutboundVlessConfig, OutboundVmessConfig,
};
