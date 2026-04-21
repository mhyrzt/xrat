#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConfigListFilter {
    pub include_deleted: bool,
    pub only_enabled: bool,
    pub only_selected: bool,
    pub only_active: bool,
    pub subscription_id: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigRecord {
    pub id: i64,
    pub subscription_id: Option<i64>,
    pub dedup_key: String,
    pub protocol: String,
    pub address: String,
    pub port: i64,
    pub username: Option<String>,
    pub uuid: Option<String>,
    pub password: Option<String>,
    pub method: Option<String>,
    pub network: String,
    pub tls: Option<String>,
    pub sni: Option<String>,
    pub host: Option<String>,
    pub path: Option<String>,
    pub name: Option<String>,
    pub is_active: bool,
    pub is_enabled: bool,
    pub is_deleted: bool,
    pub is_selected: bool,
    pub imported_at: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}
