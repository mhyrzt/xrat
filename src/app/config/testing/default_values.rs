use std::path::PathBuf;

use super::super::defaults;
use super::types::{
    ConnectionTestStage, DownloadTestSettings, GeoIpTestSettings, IcmpTestSettings,
    RealDelayTestSettings, TcpTestSettings, TestFailurePolicy, TestingSettings,
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
            country_path: PathBuf::from(defaults::DEFAULT_TEST_GEOIP_COUNTRY_PATH),
            city_path: PathBuf::from(defaults::DEFAULT_TEST_GEOIP_CITY_PATH),
            asn_path: PathBuf::from(defaults::DEFAULT_TEST_GEOIP_ASN_PATH),
        }
    }
}
