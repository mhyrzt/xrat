use std::{io, net::Ipv4Addr, num::NonZeroU32};

pub const TABLE: &str = "xrat_gateway";
pub const CAPTURE_COMMENT: &str = "xrat gateway capture";
pub const RETURN_COMMENT: &str = "xrat gateway return";
pub const CAPTURE_DROP_COMMENT: &str = "xrat gateway capture ingress drop";

pub fn capture_transaction(
    lan: NonZeroU32,
    capture: NonZeroU32,
    subnet: Ipv4Addr,
    prefix: u8,
) -> io::Result<String> {
    if lan == capture || !(1..=30).contains(&prefix) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "capture requires distinct interfaces and a valid LAN prefix",
        ));
    }
    let mask = u32::MAX << (32 - prefix);
    if u32::from(subnet) & !mask != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "gateway LAN subnet has host bits",
        ));
    }
    Ok(format!(
        "insert rule ip {TABLE} forward meta iif {capture} counter drop comment \"{CAPTURE_DROP_COMMENT}\"\n\
         insert rule ip {TABLE} forward meta iif {capture} meta oif {lan} ip daddr {subnet}/{prefix} ct state established,related counter accept comment \"{RETURN_COMMENT}\"\n\
         insert rule ip {TABLE} forward meta iif {lan} meta oif {capture} ip saddr {subnet}/{prefix} counter accept comment \"{CAPTURE_COMMENT}\"\n"
    ))
}
