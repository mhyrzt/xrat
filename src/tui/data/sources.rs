use crate::db::SubscriptionRecord;

#[derive(Debug, Clone, PartialEq)]
pub struct TuiSourceRow {
    pub id: i64,
    pub kind: String,
    pub value: String,
    pub name: Option<String>,
    pub config_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl TuiSourceRow {
    pub fn display_name(&self) -> &str {
        self.name
            .as_deref()
            .filter(|name| !name.is_empty())
            .unwrap_or("-")
    }

    pub fn value_label(&self) -> &str {
        if self.value.is_empty() {
            "-"
        } else {
            &self.value
        }
    }
}

impl From<SubscriptionRecord> for TuiSourceRow {
    fn from(value: SubscriptionRecord) -> Self {
        Self {
            id: value.id,
            kind: value.source_kind,
            value: value.source_url.unwrap_or_default(),
            name: value.name,
            config_count: value.config_count,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
