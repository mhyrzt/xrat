use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct GatewaySettings {
    pub interface: Option<String>,
    pub lan_subnet: Option<String>,
    pub local_subnets: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::super::AppConfig;

    #[test]
    fn parses_gateway_settings_and_defaults() {
        let config: AppConfig = toml::from_str("[gateway]\ninterface = 'eth0'\nlan_subnet = '192.168.1.0/24'\nlocal_subnets = ['192.168.2.0/24']\n").unwrap();
        assert_eq!(config.gateway.interface.as_deref(), Some("eth0"));
        assert_eq!(config.gateway.local_subnets, ["192.168.2.0/24"]);
        let default: AppConfig = toml::from_str("").unwrap();
        assert!(default.gateway.interface.is_none());
        assert!(default.gateway.local_subnets.is_empty());
    }
}
