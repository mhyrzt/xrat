use super::*;
use std::net::TcpListener;

pub(super) fn assign_ephemeral_inbound_ports(
    launch: &mut ResolvedLaunch,
) -> crate::app::Result<()> {
    if let Some(endpoint) = launch.endpoints.socks.as_mut() {
        endpoint.port = allocate_port(&endpoint.host)?;
    }
    if let Some(endpoint) = launch.endpoints.http.as_mut() {
        endpoint.port = allocate_port(&endpoint.host)?;
    }
    if let Some(endpoint) = launch.endpoints.shadowsocks.as_mut() {
        endpoint.port = allocate_port(&endpoint.host)?;
    }

    for inbound in &mut launch.config.inbounds {
        match inbound.tag.as_str() {
            "socks-in" => {
                if let Some(endpoint) = launch.endpoints.socks.as_ref() {
                    inbound.port = endpoint.port;
                }
            }
            "http-in" => {
                if let Some(endpoint) = launch.endpoints.http.as_ref() {
                    inbound.port = endpoint.port;
                }
            }
            "shadowsocks-in" => {
                if let Some(endpoint) = launch.endpoints.shadowsocks.as_ref() {
                    inbound.port = endpoint.port;
                }
            }
            _ => {}
        }
    }

    if let Some(endpoint) = launch.endpoints.socks.as_ref() {
        launch.ready_port = endpoint.port;
    } else if let Some(endpoint) = launch.endpoints.http.as_ref() {
        launch.ready_port = endpoint.port;
    } else if let Some(endpoint) = launch.endpoints.shadowsocks.as_ref() {
        launch.ready_port = endpoint.port;
    }

    Ok(())
}

fn allocate_port(host: &str) -> crate::app::Result<u16> {
    let bind_host = connect_host_for_bind_host(host);
    let listener = TcpListener::bind((bind_host.as_str(), 0)).map_err(|err| {
        AppError::InvalidArgument(format!("failed to allocate ephemeral inbound port: {err}"))
    })?;
    let port = listener
        .local_addr()
        .map_err(|err| {
            AppError::InvalidArgument(format!("failed to resolve ephemeral inbound port: {err}"))
        })?
        .port();
    drop(listener);
    Ok(port)
}
