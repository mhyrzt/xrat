#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeSessionStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

impl RuntimeSessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "starting" => Some(Self::Starting),
            "running" => Some(Self::Running),
            "stopping" => Some(Self::Stopping),
            "stopped" => Some(Self::Stopped),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSessionInsert {
    pub config_id: Option<i64>,
    pub status: RuntimeSessionStatus,
    pub socks_host: Option<String>,
    pub socks_port: Option<i64>,
    pub http_host: Option<String>,
    pub http_port: Option<i64>,
    pub shadowsocks_host: Option<String>,
    pub shadowsocks_port: Option<i64>,
    pub process_id: Option<i64>,
    pub started_at: Option<String>,
    pub stopped_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSessionRecord {
    pub id: i64,
    pub config_id: Option<i64>,
    pub status: RuntimeSessionStatus,
    pub socks_host: Option<String>,
    pub socks_port: Option<i64>,
    pub http_host: Option<String>,
    pub http_port: Option<i64>,
    pub shadowsocks_host: Option<String>,
    pub shadowsocks_port: Option<i64>,
    pub process_id: Option<i64>,
    pub started_at: Option<String>,
    pub stopped_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
