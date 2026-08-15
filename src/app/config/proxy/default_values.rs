use std::path::PathBuf;

use super::super::SecretString;
use super::super::defaults;
use super::types::{
    AuthSettings, FragmentSettings, HttpSettings, LogSettings, MuxSettings, NetworkSettings,
    RotationSettings, RuntimeSettings, ShadowsocksSettings, SniffingSettings, SocksSettings,
    StatsSettings,
};

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            engine: defaults::DEFAULT_RUNTIME_ENGINE.to_string(),
            replace_active_session: defaults::DEFAULT_REPLACE_ACTIVE_SESSION,
            rotation: RotationSettings::default(),
            log: LogSettings::default(),
            socks: SocksSettings::default(),
            http: HttpSettings::default(),
            shadowsocks: ShadowsocksSettings::default(),
            sniffing: SniffingSettings::default(),
            stats: StatsSettings::default(),
            mux: MuxSettings::default(),
            fragment: FragmentSettings::default(),
            network: NetworkSettings::default(),
        }
    }
}

impl Default for MuxSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_MUX_ENABLED,
            concurrency: defaults::DEFAULT_MUX_CONCURRENCY,
            xudp_concurrency: defaults::DEFAULT_MUX_XUDP_CONCURRENCY,
            xudp_proxy_udp443: defaults::DEFAULT_MUX_XUDP_PROXY_UDP443.to_string(),
        }
    }
}

impl Default for FragmentSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_FRAGMENT_ENABLED,
            packets_mode: defaults::DEFAULT_FRAGMENT_PACKETS_MODE.to_string(),
            packets: defaults::DEFAULT_FRAGMENT_PACKETS,
            length: defaults::DEFAULT_FRAGMENT_LENGTH,
            interval: defaults::DEFAULT_FRAGMENT_INTERVAL,
        }
    }
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            interface: defaults::DEFAULT_NETWORK_INTERFACE.to_string(),
            bind_address: defaults::DEFAULT_NETWORK_BIND_ADDRESS.to_string(),
            mark: defaults::DEFAULT_NETWORK_MARK,
            listen_interface: defaults::DEFAULT_NETWORK_LISTEN_INTERFACE.to_string(),
        }
    }
}

impl Default for StatsSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_STATS_ENABLED,
            host: defaults::DEFAULT_STATS_HOST.to_string(),
            port: defaults::DEFAULT_STATS_PORT,
        }
    }
}

impl Default for RotationSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_ROTATION_ENABLED,
            interval_secs: defaults::DEFAULT_ROTATION_INTERVAL_SECS,
            health_trigger_enabled: defaults::DEFAULT_ROTATION_HEALTH_TRIGGER_ENABLED,
            health_failure_threshold: defaults::DEFAULT_ROTATION_HEALTH_FAILURE_THRESHOLD,
            cooldown_secs: defaults::DEFAULT_ROTATION_COOLDOWN_SECS,
            test_concurrency: defaults::DEFAULT_ROTATION_TEST_CONCURRENCY,
            test_stages: defaults::DEFAULT_ROTATION_TEST_STAGES
                .iter()
                .map(|value| value.to_string())
                .collect(),
            refresh_subscriptions: defaults::DEFAULT_ROTATION_REFRESH_SUBSCRIPTIONS,
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
