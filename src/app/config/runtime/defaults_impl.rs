use std::path::PathBuf;

use super::super::SecretString;
use super::super::defaults;
use super::types::{
    AuthSettings, HttpSettings, LogSettings, RuntimeSettings, ShadowsocksSettings,
    SniffingSettings, SocksSettings,
};

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            engine: defaults::DEFAULT_RUNTIME_ENGINE.to_string(),
            replace_active_session: defaults::DEFAULT_REPLACE_ACTIVE_SESSION,
            log: LogSettings::default(),
            socks: SocksSettings::default(),
            http: HttpSettings::default(),
            shadowsocks: ShadowsocksSettings::default(),
            sniffing: SniffingSettings::default(),
        }
    }
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_LOG_ENABLED,
            mask: defaults::DEFAULT_LOG_MASK.to_string(),
            dir: PathBuf::from(defaults::DEFAULT_LOG_DIR),
            dns_log: defaults::DEFAULT_LOG_DNS_LOG,
            level: defaults::DEFAULT_LOG_LEVEL.to_string(),
            keep: defaults::DEFAULT_LOG_KEEP,
        }
    }
}

impl Default for SocksSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_SOCKS_ENABLED,
            host: defaults::DEFAULT_INBOUND_HOST.to_string(),
            port: defaults::DEFAULT_SOCKS_PORT,
            udp: defaults::DEFAULT_SOCKS_UDP,
            auth: AuthSettings::default(),
        }
    }
}

impl Default for HttpSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_HTTP_ENABLED,
            host: defaults::DEFAULT_INBOUND_HOST.to_string(),
            port: defaults::DEFAULT_HTTP_PORT,
        }
    }
}

impl Default for ShadowsocksSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_SHADOWSOCKS_ENABLED,
            host: defaults::DEFAULT_INBOUND_HOST.to_string(),
            port: defaults::DEFAULT_SHADOWSOCKS_PORT,
            method: defaults::DEFAULT_SHADOWSOCKS_METHOD.to_string(),
            password: SecretString::Literal(defaults::DEFAULT_SHADOWSOCKS_PASSWORD.to_string()),
            network: defaults::DEFAULT_SHADOWSOCKS_NETWORK.to_string(),
        }
    }
}

impl Default for SniffingSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_SNIFFING_ENABLED,
            dest_override: defaults::DEFAULT_SNIFFING_DEST_OVERRIDE
                .iter()
                .map(|value| value.to_string())
                .collect(),
            route_only: defaults::DEFAULT_SNIFFING_ROUTE_ONLY,
            metadata_only: defaults::DEFAULT_SNIFFING_METADATA_ONLY,
            domains_excluded: Vec::new(),
            ips_excluded: Vec::new(),
        }
    }
}
