#[derive(Clone, Debug, PartialEq)]
pub struct NewEvent {
    pub level: String,
    pub source: String,
    pub kind: String,
    pub config_id: Option<i64>,
    pub session_id: Option<i64>,
    pub message: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventRecord {
    pub id: i64,
    pub level: String,
    pub source: String,
    pub kind: String,
    pub config_id: Option<i64>,
    pub session_id: Option<i64>,
    pub message: String,
    pub detail: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EventFilter {
    pub source: Option<String>,
    /// Allowed levels (e.g. `["warn", "error"]`). `None` means all levels.
    pub levels: Option<Vec<String>>,
    pub limit: i64,
}
