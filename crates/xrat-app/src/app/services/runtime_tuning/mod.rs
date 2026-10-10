mod dns_policy;
mod dns_singbox;
pub(crate) mod dns_validation;
mod network;
pub(crate) use dns_policy::apply_xray_runtime as apply_xray_dns_runtime;
pub(crate) use dns_singbox::apply_runtime as apply_singbox_dns_runtime;
mod prelude;
mod singbox;
pub(crate) mod tun_validation;
mod xray;

#[cfg(test)]
mod dns_tests;
#[cfg(test)]
mod singbox_conformance;
#[cfg(test)]
mod tests;

pub(crate) use network::resolve_listen_interface_addr;
pub(crate) use singbox::{build_singbox_dns_options, build_singbox_routing_options};
pub(crate) use xray::{
    apply_xray_dns_options, apply_xray_routing_options, build_xray_gen_options,
    detect_xray_compatibility, detect_xray_compatibility_with_spawner,
    ensure_xray_tun_supported_with_spawner, xray_binary_version_with_spawner,
};
