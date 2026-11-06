use crate::app::config::SecretError;
use xrat_engines::singbox::SingboxRuntimeError;
use xrat_engines::xray::runtime_process::XrayRuntimeError;
use xrat_support::geoip::GeoIpError;

/// Application-facing error.
///
/// Layer-owned error types (`DbError`, `GeoIpError`, `XrayRuntimeError`,
/// `SingboxRuntimeError`) are wrapped as typed variants so the application layer
/// never fabricates their messages. Infrastructure library errors (`io`,
/// `toml`, `reqwest`, `serde_json`) are still adapted here; these are the
/// remaining direct library couplings and are targeted by the HTTP/filesystem
/// port work.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("application I/O failed")]
    Io(#[from] std::io::Error),

    #[error("failed to parse application config: {0}")]
    ConfigToml(#[from] toml::de::Error),

    #[error(transparent)]
    Database(#[from] xrat_db::DbError),

    #[error(transparent)]
    Decode(#[from] xrat_support::decode::DecodeError),

    #[error("HTTP request failed: {0}")]
    Http(#[from] xrat_support::http::HttpError),

    #[error(
        "{operation} failed: {source}. Check access to GitHub; if needed, retry with HTTPS_PROXY or ALL_PROXY set to your working proxy."
    )]
    ReleaseHttp {
        operation: &'static str,
        #[source]
        source: xrat_support::http::HttpError,
    },

    #[error("JSON serialization failed")]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Secret(#[from] SecretError),

    #[error(transparent)]
    Geoip(#[from] GeoIpError),

    #[error(transparent)]
    XrayRuntime(#[from] XrayRuntimeError),

    #[error(transparent)]
    XraySignal(#[from] xrat_engines::xray::runtime_process::XraySignalError),

    #[error(transparent)]
    SingboxRuntime(#[from] SingboxRuntimeError),

    #[error(transparent)]
    SingboxVersion(#[from] xrat_engines::singbox::SingboxVersionError),

    #[error("no supported config found in input")]
    NoSupportedConfig,

    #[error("add accepts exactly one config URI/text")]
    MultipleConfigsForAdd,

    #[error(
        "JSON is not a proxy share link; use local import for complete Xray profiles (SIP008 subscriptions are unsupported)"
    )]
    RawJsonImportUnsupported,

    #[error("could not determine XRAT home directory")]
    MissingHomeDirectory,

    #[error("[database.postgres].user is required when backend = \"postgres\"")]
    MissingPostgresUser,

    #[error("[database.postgres].db_name is required when backend = \"postgres\"")]
    MissingPostgresDatabaseName,

    #[error("unsupported protocol in database: {0}")]
    UnsupportedProtocol(String),

    #[error("{0}")]
    InvalidArgument(String),

    #[error("unsupported platform: {0}")]
    UnsupportedPlatform(String),

    #[error("MMDB download failed for {edition} from {url}: {reason}")]
    GeoipDownload {
        edition: String,
        url: String,
        reason: String,
    },

    #[error(
        "runtime session already active; disconnect first or enable [runtime].replace_active_session"
    )]
    RuntimeSessionAlreadyActive,

    #[error(
        "no local runtime inbound is enabled; enable [runtime.socks], [runtime.http], or [runtime.shadowsocks]"
    )]
    NoRuntimeInboundEnabled,

    #[error("background task failed")]
    TaskJoin(#[from] tokio::task::JoinError),
}

pub type Result<T> = std::result::Result<T, AppError>;
