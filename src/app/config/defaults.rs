pub const DEFAULT_RUNTIME_ENGINE: &str = "xray";
pub const DEFAULT_REPLACE_ACTIVE_SESSION: bool = true;

pub const DEFAULT_LOG_ENABLED: bool = true;
pub const DEFAULT_LOG_MASK: &str = "none";
pub const DEFAULT_LOG_DIR: &str = "logs";
pub const DEFAULT_LOG_DNS_LOG: bool = false;
pub const DEFAULT_LOG_LEVEL: &str = "warning";
pub const DEFAULT_LOG_KEEP: bool = true;

pub const DEFAULT_INBOUND_HOST: &str = "0.0.0.0";
pub const DEFAULT_SOCKS_ENABLED: bool = true;
pub const DEFAULT_SOCKS_PORT: u16 = 1080;
pub const DEFAULT_SOCKS_UDP: bool = true;
pub const DEFAULT_HTTP_ENABLED: bool = false;
pub const DEFAULT_HTTP_PORT: u16 = 8080;

pub const DEFAULT_SHADOWSOCKS_ENABLED: bool = false;
pub const DEFAULT_SHADOWSOCKS_PORT: u16 = 1081;
pub const DEFAULT_SHADOWSOCKS_METHOD: &str = "aes-128-gcm";
pub const DEFAULT_SHADOWSOCKS_PASSWORD: &str = "change-me";
pub const DEFAULT_SHADOWSOCKS_NETWORK: &str = "tcp,udp";

pub const DEFAULT_SNIFFING_ENABLED: bool = true;
pub const DEFAULT_SNIFFING_DEST_OVERRIDE: &[&str] = &["http", "tls", "quic"];
pub const DEFAULT_SNIFFING_ROUTE_ONLY: bool = true;
pub const DEFAULT_SNIFFING_METADATA_ONLY: bool = false;

pub const DEFAULT_ROUTING_DOMAIN_STRATEGY: &str = "IPIfNonMatch";

pub const DEFAULT_GEO_AUTO_UPDATE: bool = false;
pub const DEFAULT_GEO_UPDATE_INTERVAL_HOURS: u64 = 168;

pub const DEFAULT_DNS_QUERY_STRATEGY: &str = "UseSystem";
pub const DEFAULT_DNS_USE_SYSTEM_HOSTS: bool = true;
pub const DEFAULT_DNS_DISABLE_CACHE: bool = false;
pub const DEFAULT_DNS_DISABLE_FALLBACK: bool = false;
pub const DEFAULT_DNS_ENABLE_PARALLEL_QUERY: bool = true;

pub const DEFAULT_ICMP_TIMEOUT_MS: u64 = 3000;
pub const DEFAULT_TESTING_CONCURRENCY: i32 = 0;
pub const DEFAULT_TEST_STAGE_ENABLED: bool = true;
pub const DEFAULT_DOWNLOAD_TEST_ENABLED: bool = false;
pub const DEFAULT_ICMP_ATTEMPTS: u32 = 3;
pub const DEFAULT_TCP_TIMEOUT_MS: u64 = 5000;
pub const DEFAULT_REAL_DELAY_TEST_URL: &str = "https://www.gstatic.com/generate_204";
pub const DEFAULT_REAL_DELAY_TIMEOUT_MS: u64 = 10_000;
pub const DEFAULT_XRAY_STARTUP_TIMEOUT_MS: u64 = 5000;
pub const DEFAULT_DOWNLOAD_TEST_URL: &str = "https://cachefly.cachefly.net/50mb.test";
pub const DEFAULT_DOWNLOAD_TIMEOUT_MS: u64 = 30_000;
pub const DEFAULT_UPLOAD_TIMEOUT_MS: u64 = 30_000;
pub const DEFAULT_UPLOAD_PAYLOAD_BYTES: usize = 1_000_000;
pub const DEFAULT_TEST_GEOIP_ENABLED: bool = false;
pub const DEFAULT_TEST_GEOIP_COUNTRY_PATH: &str = "geoip/GeoLite2-Country.mmdb";
pub const DEFAULT_TEST_GEOIP_CITY_PATH: &str = "geoip/GeoLite2-City.mmdb";
pub const DEFAULT_TEST_GEOIP_ASN_PATH: &str = "geoip/GeoLite2-ASN.mmdb";
