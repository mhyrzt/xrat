mod extensions;
mod generator;
mod outbound;
mod routing;
mod stream;
mod tuning;
mod types;

use xrat_model::Node;

pub use routing::{
    XrayRouteList, XrayRoutingOptions, XrayTunSplitMode, XrayTunSplitOptions,
    enable_tun_split_routing,
};
pub use tuning::{
    FragmentOptions, MuxOptions, XrayCompatibilityPolicy, XrayCompatibilityTarget, XrayGenOptions,
};
pub use types::{
    GrpcSettings, HttpUpgradeSettings, Inbound, KcpSettings, LogConfig, Mux, Outbound, RawSettings,
    RealitySettings, RoutingConfig, RoutingRule, Sockopt, StreamSettings, TlsSettings, WsSettings,
    XhttpSettings, XrayConfig, XrayDnsConfig, XrayDnsHostValue,
};

mod parse;
pub use parse::enable_stats_api;
pub use parse::generate_probe_config;
pub use parse::generate_probe_config_with_options;
pub use parse::generate_runtime_config;
pub use parse::generate_runtime_config_for_inbounds;
pub use parse::generate_runtime_config_for_inbounds_with_options;
pub use parse::generate_runtime_config_with_inbounds;
pub use parse::{XrayTunCaptureOptions, enable_tun_capture};
