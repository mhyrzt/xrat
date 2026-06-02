use crate::app::config::SecretError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("application I/O failed")]
    Io(#[from] std::io::Error),

    #[error("failed to parse application config")]
    ConfigToml(#[from] toml::de::Error),

    #[error(transparent)]
    Database(#[from] crate::db::DbError),

    #[error(transparent)]
    Decode(#[from] crate::support::decode::DecodeError),

    #[error("HTTP request failed")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization failed")]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Secret(#[from] SecretError),

    #[error("no supported config found in input")]
    NoSupportedConfig,

    #[error("add accepts exactly one config URI/text")]
    MultipleConfigsForAdd,

    #[error("raw JSON config import is not persisted yet; provide subscription links/text instead")]
    RawJsonImportUnsupported,

    #[error("could not determine XRAT home directory")]
    MissingHomeDirectory,

    #[error("[database.postgres].user is required when backend = \"postgres\"")]
    MissingPostgresUser,

    #[error("[database.postgres].db_name is required when backend = \"postgres\"")]
    MissingPostgresDatabaseName,

    #[error("unsupported protocol in database: {0}")]
    UnsupportedProtocol(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("geoip download failed for {edition} from {url}: {reason}")]
    GeoipDownload {
        edition: String,
        url: String,
        reason: String,
    },

    #[error("failed to spawn Xray process: {0}")]
    XraySpawn(String),

    #[error("Xray process exited during startup: {0}")]
    XrayExited(String),

    #[error("Xray did not open local port {port} before startup timeout")]
    XrayStartupTimeout { port: u16 },

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
