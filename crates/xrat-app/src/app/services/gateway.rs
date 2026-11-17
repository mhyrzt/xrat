use std::net::Ipv4Addr;

use serde::Serialize;

use crate::app::{AppError, Result, config::GatewaySettings};

mod preflight;
pub use preflight::{Report, inspect};

#[derive(Default)]
pub struct GatewayOverrides {
    pub interface: Option<String>,
    pub lan_subnet: Option<String>,
    pub local_subnets: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ValueWithSource<T> {
    pub value: T,
    pub source: &'static str,
}

#[derive(Debug, Serialize)]
pub struct EffectiveSettings {
    pub interface: ValueWithSource<Option<String>>,
    pub lan_subnet: ValueWithSource<Option<String>>,
    pub local_subnets: ValueWithSource<Vec<String>>,
}

pub fn resolve(settings: &GatewaySettings, args: &GatewayOverrides) -> Result<EffectiveSettings> {
    let interface = select(args.interface.clone(), settings.interface.clone());
    let lan_subnet = select(args.lan_subnet.clone(), settings.lan_subnet.clone());
    let local_subnets = if args.local_subnets.is_empty() {
        ValueWithSource {
            value: settings.local_subnets.clone(),
            source: if settings.local_subnets.is_empty() {
                "default"
            } else {
                "config"
            },
        }
    } else {
        ValueWithSource {
            value: args.local_subnets.clone(),
            source: "cli",
        }
    };
    if let Some(name) = &interface.value
        && (name.is_empty()
            || name.len() > 15
            || name == "."
            || name == ".."
            || name.bytes().any(|byte| {
                byte == b'/'
                    || byte == b':'
                    || byte.is_ascii_whitespace()
                    || byte.is_ascii_control()
            }))
    {
        return Err(AppError::InvalidArgument(
            "gateway interface must be a valid Linux interface name".into(),
        ));
    }
    if let Some(subnet) = &lan_subnet.value {
        validate_subnet(subnet)?;
    }
    for subnet in &local_subnets.value {
        validate_subnet(subnet)?;
    }
    Ok(EffectiveSettings {
        interface,
        lan_subnet,
        local_subnets,
    })
}

fn select(cli: Option<String>, config: Option<String>) -> ValueWithSource<Option<String>> {
    if cli.is_some() {
        ValueWithSource {
            value: cli,
            source: "cli",
        }
    } else if config.is_some() {
        ValueWithSource {
            value: config,
            source: "config",
        }
    } else {
        ValueWithSource {
            value: None,
            source: "undetermined",
        }
    }
}

fn validate_subnet(value: &str) -> Result<()> {
    let invalid = || {
        AppError::InvalidArgument(format!(
            "invalid IPv4 network CIDR '{value}'; use a network address such as 192.168.1.0/24"
        ))
    };
    let (address, prefix) = value.split_once('/').ok_or_else(invalid)?;
    let address: Ipv4Addr = address.parse().map_err(|_| invalid())?;
    let prefix: u8 = prefix.parse().map_err(|_| invalid())?;
    if prefix > 32 {
        return Err(invalid());
    }
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    if u32::from(address) & !mask != 0 {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overrides_replace_config_and_preserve_sources() {
        let config = GatewaySettings {
            interface: Some("eth0".into()),
            lan_subnet: Some("192.168.1.0/24".into()),
            local_subnets: vec!["192.168.2.0/24".into()],
        };
        let args = GatewayOverrides {
            interface: Some("wlan0".into()),
            local_subnets: vec!["10.0.0.0/8".into()],
            ..Default::default()
        };
        let resolved = resolve(&config, &args).unwrap();
        assert_eq!(resolved.interface.value.as_deref(), Some("wlan0"));
        assert_eq!(resolved.interface.source, "cli");
        assert_eq!(resolved.lan_subnet.source, "config");
        assert_eq!(resolved.local_subnets.value, ["10.0.0.0/8"]);
        assert_eq!(resolved.local_subnets.source, "cli");
    }

    #[test]
    fn rejects_invalid_cidrs_and_interface_names() {
        for subnet in ["192.168.1.1/24", "10.0.0.0/33", "::1/128", "invalid"] {
            let args = GatewayOverrides {
                lan_subnet: Some(subnet.into()),
                ..Default::default()
            };
            assert!(resolve(&GatewaySettings::default(), &args).is_err());
        }
        let args = GatewayOverrides {
            interface: Some("bad/name".into()),
            ..Default::default()
        };
        assert!(resolve(&GatewaySettings::default(), &args).is_err());
    }
}
