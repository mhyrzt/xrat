#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceKind {
    Url,
    File,
    RawText,
}

impl SourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Url => "url",
            Self::File => "file",
            Self::RawText => "raw_text",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ImportSource {
    pub kind: SourceKind,
    pub value: String,
    pub name: Option<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImportSummary {
    pub subscription_id: i64,
    pub imported_configs: usize,
    pub removed_configs: u64,
    pub total_configs: i64,
}

/// A URL-backed subscription eligible for automatic refresh.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefreshableSubscription {
    pub id: i64,
    pub source_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubscriptionRecord {
    pub id: i64,
    pub source_kind: String,
    pub source_url: Option<String>,
    pub name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub config_count: i64,
}
