use std::path::PathBuf;

use super::super::defaults;
use super::types::{
    ConnectionTestStage, DownloadTestSettings, GeoIpBackend, GeoIpCacheSettings,
    GeoIpRemoteProvider, GeoIpTestSettings, IcmpTestSettings, RealDelayTestSettings,
    RemoteGeoIpSettings, TcpTestSettings, TestFailurePolicy, TestingSettings,
};

impl Default for TestingSettings {
    fn default() -> Self {
        Self {
            concurrency: defaults::DEFAULT_TESTING_CONCURRENCY,
            order: default_connection_test_order(),
            failure_policy: TestFailurePolicy::Continue,
            real_delay: RealDelayTestSettings::default(),
            icmp: IcmpTestSettings::default(),
            download: DownloadTestSettings::default(),
            tcp: TcpTestSettings::default(),
            geoip: GeoIpTestSettings::default(),
        }
    }
}

impl Default for RealDelayTestSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_TEST_STAGE_ENABLED,
            url: defaults::DEFAULT_REAL_DELAY_TEST_URL.to_string(),
            timeout: defaults::DEFAULT_REAL_DELAY_TIMEOUT_MS,
        }
    }
}

fn default_connection_test_order() -> Vec<ConnectionTestStage> {
    vec![
        ConnectionTestStage::Icmp,
        ConnectionTestStage::RealDelay,
        ConnectionTestStage::Download,
    ]
}

impl Default for DownloadTestSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_DOWNLOAD_TEST_ENABLED,
            url: defaults::DEFAULT_DOWNLOAD_TEST_URL.to_string(),
            timeout: defaults::DEFAULT_DOWNLOAD_TIMEOUT_MS,
        }
    }
}

impl Default for IcmpTestSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_TEST_STAGE_ENABLED,
            attempts: defaults::DEFAULT_ICMP_ATTEMPTS,
            timeout: defaults::DEFAULT_ICMP_TIMEOUT_MS,
        }
    }
}

impl Default for TcpTestSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_TEST_STAGE_ENABLED,
            timeout: defaults::DEFAULT_TCP_TIMEOUT_MS,
        }
    }
}

impl Default for GeoIpTestSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_TEST_GEOIP_ENABLED,
            backend: GeoIpBackend::Mmdb,
            fallback: GeoIpBackend::None,
            country_path: PathBuf::from(defaults::DEFAULT_TEST_GEOIP_COUNTRY_PATH),
            city_path: PathBuf::from(defaults::DEFAULT_TEST_GEOIP_CITY_PATH),
            asn_path: PathBuf::from(defaults::DEFAULT_TEST_GEOIP_ASN_PATH),
            remote: RemoteGeoIpSettings::default(),
            cache: GeoIpCacheSettings::default(),
        }
    }
}

impl Default for RemoteGeoIpSettings {
    fn default() -> Self {
        Self {
            provider: GeoIpRemoteProvider::IpWhois,
            endpoint: String::new(),
            timeout_ms: 5000,
            api_key: String::new(),
            rate_limit_per_minute: 30,
        }
    }
}

impl Default for GeoIpCacheSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl_secs: 86_400,
            max_entries: 10_000,
        }
    }
}
