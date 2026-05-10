mod generate;
mod outbound;
mod stream;
mod types;

use crate::model::Node;

pub use types::{
    GrpcSettings, Inbound, LogConfig, Outbound, StreamSettings, TcpSettings, TlsSettings,
    WsSettings, XrayConfig,
};

pub fn generate_probe_config(node: &Node, local_port: u16) -> Result<XrayConfig, String> {
    generate::generate_probe_config(node, local_port)
}

pub fn generate_runtime_config(
    node: &Node,
    socks_port: u16,
    http_port: Option<u16>,
) -> Result<XrayConfig, String> {
    generate::generate_runtime_config(node, socks_port, http_port)
}

pub fn generate_runtime_config_with_inbounds(
    node: &Node,
    socks_host: &str,
    socks_port: u16,
    http_host: Option<&str>,
    http_port: Option<u16>,
) -> Result<XrayConfig, String> {
    generate::generate_runtime_config_with_inbounds(
        node, socks_host, socks_port, http_host, http_port,
    )
}

pub fn generate_runtime_config_for_inbounds(
    node: &Node,
    socks: Option<(&str, u16, bool)>,
    http: Option<(&str, u16)>,
) -> Result<XrayConfig, String> {
    generate::generate_runtime_config_for_inbounds(node, socks, http)
}
