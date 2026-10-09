use serde::{Deserialize, Serialize};

use xrat_model::{Node, Protocol};

mod hy2;
mod simple;
mod transport;
mod trojan;
mod vless;
mod vmess;

#[cfg(test)]
mod tests;

mod builder;
pub use builder::SingboxCacheFile;
pub use builder::SingboxClashApi;
pub use builder::SingboxConfig;
pub use builder::SingboxDnsConfig;
pub use builder::SingboxExperimental;
pub use builder::SingboxInbound;
pub use builder::SingboxInboundUser;
pub use builder::SingboxRouteList;
pub use builder::SingboxRoutingOptions;
pub use builder::SingboxTunOptions;
pub use builder::SingboxTunSplitMode;
pub use builder::SingboxTunSplitOptions;
pub use builder::generate_singbox_probe_config;
pub use builder::generate_singbox_runtime_config;
pub use builder::generate_singbox_runtime_config_with_dns;
pub use types::SingboxLogConfig;
pub use types::SingboxRoute;

mod types;
